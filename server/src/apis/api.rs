use std::sync::Arc;
use crate::apis::error::{ApiError, ApiSessionError, ToResponse};
use crate::data::{SessionRequest};
use crate::services::db::MongoServer;
use crate::services::game::{CreateLobby, FileMetadata};
use crate::services::lobby::CanJoinLobby;
use actix::Addr;

use actix_multipart::Multipart;
use actix_web::{get, HttpRequest, HttpResponse, post, web};
use bson::Bson;
use chrono::Local;
use cult_common::dto::api::{ApiResponse};
use cult_common::dto::file::{FileMultiPart};
use cult_common::wasm_lib::hashs::validate::ValidateHash;
use cult_common::wasm_lib::JeopardyMode;
use futures::{AsyncWriteExt, StreamExt};
use mongodb::gridfs::GridFsUploadStream;
use serde::Serialize;
use serde_json::json;
use cult_common::backend::JeopardyBoard;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use utoipa::ToSchema;
use crate::apis::data::{extract_header_string, extract_value, get_internal_server_error_json, get_lobby_id_from_header, get_session, get_session_with_token_update_or_create_new, set_session_token_cookie};
use crate::authentication::discord::is_admin;
use crate::services::game;
use crate::services::game::UserSession;

#[utoipa::path(
    get,
    path = "/api/info",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
        ("lobby-id" = String, Header, description = "Lobby ID")
    ),
    responses(
        (status = 200, description = "User session retrieved successfully", body = UserSessionWithAdmin),
        (status = 404, description = "No User Session", body = ApiSessionError),
        (status = 500, description = "No Lobby found", body = ApiError)
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[get("/api/info")]
async fn game_info(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    println!("{:?}", extract_value(&req, "key"));

    let lobby_id = match extract_header_string(&req, "lobby-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };
    let lobby = srv.send(game::LobbyExists { lobby_id: LobbyId::of(lobby_id.clone())}).await.expect("No Lobby found!");
    let error = json!(
        {
            "Error": "Lobby not found",
            "Lobby": lobby_id
        }
    );
    let user =match lobby {
        false => return Ok(HttpResponse::from(HttpResponse::InternalServerError().json(error))),
        true => "Found something"
    };
    Ok(HttpResponse::from(HttpResponse::Ok().json(user)))
}
#[derive(Debug, Clone,Serialize, ToSchema)]
pub struct UserSessionWithAdmin{
    user_session:UserSession,
    is_admin:bool
}


#[utoipa::path(
    get,
    path = "/api/session",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token")
    ),
    responses(
        (status = 200, description = "User session retrieved successfully", body = UserSessionWithAdmin),
        (status = 404, description = "No User Session", body = ApiSessionError)
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
    

)]
#[get("/api/session")]
async fn api_session_request(req: HttpRequest ,db:web::Data<Arc<MongoServer>> ) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(ApiError::Session(ApiSessionError::SessionNotFound).to_response()),
    };
    let is_admin = is_admin(&user_session, &db).await;
    let mut response = HttpResponse::from(HttpResponse::Ok().json(UserSessionWithAdmin{user_session:user_session.clone(), is_admin}));
    set_session_token_cookie(&mut response,  &user_session);
    Ok(response)
}







#[post("/api/upload/filepart")]
async fn upload_file_part(req: HttpRequest,db: web::Data<Arc<MongoServer>>,mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(HttpResponse::InternalServerError().json("No User Session")),
    };
    if is_admin(&user_session, &db).await == false {
        return Ok(file_part_error(None, "No Admin").await)
    } 
    let discord_id = match user_session.discord_auth.clone() {
        Some(data) => data.discord_user.unwrap().discord_id,
        None => return Ok(file_part_error(None, "No Discord ID").await)};

    let file_name = match extract_header_string(&req, "file-name") {
        Ok(data) => data,
        Err(_) => return Ok(HttpResponse::BadRequest().json("No file name provided")),
    };

    println!("UPLOAD FILE Part");
    let start_time: chrono::DateTime<Local> = Local::now();
    let mut file_multi = FileMultiPart::default();

    let mut upload_stream = None;

    file_multi.file_name = Some(file_name.clone());

    if let Ok(data) = db.collections.file_bucket_files.find_one(bson::doc!{"filename":Some(file_name.clone())}).await {
        if let Some(_) = data {
            return Ok(file_part_error(upload_stream, "Pre File already exists").await);
        }
    }







    file_multi.uploader_id = Some(discord_id);


    while let Some(item) = payload.next().await {
        match item {
            Ok(mut field) => {
                let name = match field.name() {
                    Some(name) => name.to_owned(),
                    None => return Ok(file_part_error(upload_stream, "Can´t get name").await),
                };
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(data) => data,
                        Err(_) => return Ok(file_part_error(upload_stream, "Can´t get chunk").await),
                    };

                    match name.as_str() {
                        "file_data" => {
                            if upload_stream.is_none() {
                                let file_name = match &file_multi.file_name {
                                    Some(data) => data,
                                    None => return Ok(file_part_error(None, "Can´t get file name").await),
                                };
                                let file_type = match field.content_type() {
                                    Some(data) => data,
                                    None => return Ok(file_part_error(None, "Can´t get content type").await),
                                };
                                file_multi.file_type = Some(file_type.to_string()); 
                                let mut stream = db.collections.file_bucket.open_upload_stream(file_name.to_string()).await.expect("Can´t open upload stream");   
                                stream.write_all(&data).await.expect("Can´t write to upload stream");
                                upload_stream = Some(stream);
                            } else {
                                let stream = match &mut upload_stream {
                                    Some(data) => data,
                                    None => return Ok(file_part_error(None, "Can´t get upload stream").await),                         
                                 };
                                stream.write_all(&data).await.expect("Can´t write to upload stream");
                            }                        
                        },
                        "validate_hash" => {
                            file_multi.validate_hash = Some(ValidateHash::new(String::from_utf8(data.to_vec()).expect("Can´t convert to string")));
                        }
                        _ => {
                            println!("Field not found {:?}", name);

                        }
                    }
                }
            }
            Err(_) => return Ok(file_part_error(upload_stream, "Can´t get file part").await),
        }
    }

    println!("UPLOAD FILE PART size: {:?} in {:?}", file_multi.file_name, Local::now().signed_duration_since(start_time));
    println!("{:#?}", file_multi);
    println!("{:#?}", upload_stream.is_none());

    if !file_multi.is_valid() {
        return Ok(file_part_error(upload_stream, "File Multi Part is not valid").await);
    }

    if let Some(mut stream) = upload_stream {
        let id = stream.id().clone();

        if let Ok(data) = db.collections.file_bucket_files.find_one(bson::doc!{"filename":Some(file_name.clone())}).await {
            if let Some(_) = data {
                return Ok(file_part_error(Some(stream), "After File already exists").await);
            }
        }
        if let Err(_) = stream.flush().await {
            return Ok(file_part_error(Some(stream), "Can´t flush upload stream").await);
        }
        if let Err(_) = stream.close().await {
            return Ok(file_part_error(Some(stream), "Can´t close upload stream").await);
        }

        let file_meta = FileMetadata{
            file_type: file_multi.file_type.unwrap(),
            validate_hash: file_multi.validate_hash.unwrap(),
            uploader: file_multi.uploader_id.unwrap(),
        };
        println!("{:#?}", file_meta);

        if let Err(_) = db.collections.file_bucket_files.update_one(bson::doc! { "_id":id}, bson::doc! {"$set": {"metadata": file_meta}}).await {
            return Ok(file_part_error(Some(stream), "Can´t update file type").await);
        }
    }

    return Ok(HttpResponse::Ok().finish())


}


impl From<FileMetadata> for Bson {
    fn from(data: FileMetadata) -> Self {
        let doc = bson::doc! {
            "file_type": data.file_type,
            "validate_hash": {
                "hash": data.validate_hash.get_hash(),
            },
            "uploader": {
                "id": data.uploader.id,
            },
        };
        Bson::Document(doc)
    }
}






pub async fn file_part_error(stream: Option<GridFsUploadStream>,error:&str) -> HttpResponse {
    if let Some(mut stream) = stream {
        match stream.close().await {
            Ok(data) => data,
            Err(_) => return HttpResponse::InternalServerError().json("Stream was not closing"),
        };
    }
    let response = HttpResponse::InternalServerError().json(json!({
        "Error": error
    }));
    response
}


pub fn session_error(user_session:&UserSession, str:&str) -> HttpResponse {
    let mut response = HttpResponse::InternalServerError().json(json!({
        "Error": str
    }));
    set_session_token_cookie(&mut response,  &user_session);
    response
}



#[get("/api/session-data")]
async fn session_data_request(req: HttpRequest, db: web::Data<Arc<MongoServer>>) -> Result<HttpResponse, actix_web::Error> {
    println!("SESSION DATA REQUEST");
    let user_session = get_session_with_token_update_or_create_new(&req, &db).await;
    let sessionrequest : SessionRequest = SessionRequest{
     user_session_id: user_session.user_session_id.clone(), 
     session_token: user_session.session_token.clone() 
    };

    let mut response = HttpResponse::from(HttpResponse::Ok().json(sessionrequest));
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}







#[get("/api/authorization")]
async fn has_authorization(req: HttpRequest, db: web::Data<Arc<MongoServer>>) -> HttpResponse {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return HttpResponse::InternalServerError().json("No User Session"),
    };
    let admin = is_admin(&user_session, &db).await;
    let mut response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::new(admin)));
    set_session_token_cookie(&mut response, &user_session);
    response
}

#[get("/api/discord_session")]
async fn discord_session(req: HttpRequest, db: web::Data<Arc<MongoServer>>) -> HttpResponse {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return HttpResponse::InternalServerError().json("No User Session"),
    };
    let discord_user = match user_session.clone().discord_auth{
        None => None,
        Some(data) => data.discord_user,
    };
    let mut response = HttpResponse::from(HttpResponse::Ok().json(discord_user));
    set_session_token_cookie(&mut response, &user_session);
    response
}

#[post("/api/create")]
async fn create_game_lobby(req: HttpRequest,json: web::Json<Option<JeopardyBoard>>, srv: web::Data<Addr<game::GameServer>>, db: web::Data<Arc<MongoServer>>) -> HttpResponse {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return HttpResponse::InternalServerError().json("No User Session"),
    };
    let mut response =   HttpResponse::from(HttpResponse::NotFound());
    if is_admin(&user_session, &db).await{
        if let Some(discord_data) = user_session.clone().discord_auth {
            if let Some(discord_user) = discord_data.discord_user {
                let data = srv.send(CreateLobby { user_session_id: user_session.user_session_id.clone(), discord_id: discord_user.discord_id, jeopardy_board: json.into_inner() }).await.expect("Something happens by getting the user");
                response = HttpResponse::from(HttpResponse::Ok().json(data));
            }
        }
    }
    set_session_token_cookie(&mut response, &user_session);
    response
}


#[get("/api/join")]
async fn join_game(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>, db: web::Data<Arc<MongoServer>>) -> Result<HttpResponse, actix_web::Error> {
    println!("{:?}", extract_header_string(&req, "lobby-id"));
    let lobby_id = match get_lobby_id_from_header(&req){
        Some(data) => data,
        None => return Ok(get_internal_server_error_json(json!({"Error": "No Lobby ID Found"}))),
    };
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(HttpResponse::InternalServerError().json("No User Session")),
    };
    let lobby_adrr = match srv.send(game::LobbyAddrRequest{lobby_id:lobby_id.clone()}).await.expect("No Lobby found!") {
        Some(data) => data,
        None => return Ok(session_error(&user_session, "No Lobby Found")),
    };



    let can_join = lobby_adrr.send(CanJoinLobby { user_session_id: user_session.user_session_id.clone()}).await.expect("No Lobby found!");

    let mut response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::new(can_join)));
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}

#[get("/api/board")]
async fn board() -> HttpResponse {
    let response = HttpResponse::from(HttpResponse::Ok().json(JeopardyBoard::default(JeopardyMode::NORMAL)));
    response
}
