use std::any;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::sleep;
use crate::data::{FileChunk, SessionRequest};
use crate::main;
use crate::servers::db::{DBDatabase, FileMetadata, MongoServer};
use crate::servers::game::{CreateLobby, SessionToken};
use crate::servers::lobby::CanJoinLobby;
use actix::Addr;

use actix_multipart::form::MultipartForm;
use actix_multipart::Multipart;
use actix_web::body::MessageBody;
use actix_web::cookie::Cookie;
use actix_web::{get, HttpRequest, HttpResponse, post, web};
use attohttpc::body::File;
use bytes::Bytes;
use chrono::Local;
use cult_common::dto::api::{ApiResponse, DTOFileToken, FileDataReponse};
use cult_common::dto::{DTOFileChunk, DTOFileData};
use cult_common::wasm_lib::hashs::validate::ValidateHash;
use cult_common::wasm_lib::ids::discord::DiscordID;
use cult_common::wasm_lib::{JeopardyMode, FileData};
use futures::{AsyncWriteExt, Stream, StreamExt};
use mongodb::change_stream::session;
use oauth2::http::header::COOKIE;
use oauth2::http::{response, HeaderValue};
use serde::Serialize;
use serde_json::json;
use cult_common::backend::JeopardyBoard;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use crate::apis::data::{extract_header_string, extract_value, get_internal_server_error_json, get_lobby_id_from_header};
use crate::authentication::discord::is_admin;
use crate::servers::authentication::AuthenticationServer;
use crate::servers::{db, game};
use crate::servers::game::UserSession;

use super::data;


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

#[derive(Debug, Clone,Serialize)]
struct UserSessionWithAdmin{
    user_session:UserSession,
    is_admin:bool
}

#[get("/api/session")]
async fn api_session_request(req: HttpRequest ,db:web::Data<Arc<MongoServer>> ) -> Result<HttpResponse, actix_web::Error> {
    let user_session = get_session_or_create_new(&req, &db).await;
    let is_admin = is_admin(&user_session, &db).await;
    let mut response = HttpResponse::from(HttpResponse::Ok().json(UserSessionWithAdmin{user_session:user_session.clone(), is_admin}));
    set_session_token_cookie(&mut response,  &user_session);
    Ok(response)
}


fn file_data_response_send(req: &HttpRequest, user_session:&UserSession, file_upload_response:&FileDataReponse) -> HttpResponse {
    let mut response = match file_upload_response {
        FileDataReponse::Successful(_) => HttpResponse::from(HttpResponse::Ok().json(file_upload_response)),
        FileDataReponse::Failed(_) =>  HttpResponse::InternalServerError().json(file_upload_response)
    };
    set_session_token_cookie(&mut response,  &user_session);
    response
}


#[post("/api/upload/filedata")]
async fn upload_file_data(req: HttpRequest,  db: web::Data<Arc<MongoServer>>,  json: web::Json<DTOFileData>) -> Result<HttpResponse, actix_web::Error> {
    println!("UPLOAD FILE DATA");
    let user_session = get_session_or_create_new(&req, &db).await;
    if is_admin(&user_session, &db).await == false {
        return Ok(file_data_response_send(&req, &user_session, &FileDataReponse::Failed("No Admin".to_string())));
    } 

    let dto_filedata = json.into_inner();
    println!("DATA HASH {:?} ", dto_filedata.validate_hash);

    if db.is_file_data(&dto_filedata.file_name).await {
        return Ok(file_data_response_send(&req, &user_session, &FileDataReponse::Failed("File already exists".to_string())));
    }

    let dicord_id = match user_session.discord_auth.clone() {
        Some(data) => data.discord_user.unwrap().discord_id,
        None => return Ok(file_data_response_send(&req, &user_session, &FileDataReponse::Failed("No Discord User".to_string()))),
    };



    let file: FileData = dto_filedata.clone().to_file_data(&dicord_id);
    if !db.add_file_data(file.clone()).await {
        return Ok(file_data_response_send(&req, &user_session, &FileDataReponse::Failed("Can´t add file data".to_string())));
    }
    let mut response = file_data_response_send(&req, &user_session, &FileDataReponse::Successful(file.file_token.to_dto_file_token()));
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}







#[post("/api/upload/filechunk3")]
async fn upload_file_chunk3(req: HttpRequest,db: web::Data<Arc<MongoServer>>,mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {
    println!("UPLOAD FILE CHUNK 3");
    let start_time: chrono::DateTime<Local> = Local::now();
    //print!("SOMETHINGS HERE!{:?}", payload.next().await);
    // print size of multipart stream
    let mut size = 0;
    while let Some(chunk) = payload.next().await {
        let data = match chunk {
            Ok(data) => data,
            Err(e) => {
                println!("Error reading chunk: {}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };
        size += data.len();
    }
    println!("Size: {:?}", size);
    return Ok(HttpResponse::Ok().finish())/* 
    let file_name = match get_file_name_from_value(&req) {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File name not found"))),
    };
    let form = match payload.next().await {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File data not found"))),
    };

    //Convert the form to a FileDataForm
    let mut form = match form {
        Ok(data) => data,
        Err(e) => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File data not found"))),
    };

    while let Some(chunk) = form.next().await {
        let data = match chunk {
            Ok(data) => data,
            Err(e) => {
                println!("Error reading chunk: {}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };
        println!("Chunk: {:?}", data);
    }
    
   /* 




    let mut upload_stream = db.collections.file_bucket.open_upload_stream(file_name.clone()).await.expect("Can´t open upload stream");
    let body = form.file_data;
    let mut body = body.data;
    upload_stream.write_all(&body).await.expect("Can´t write to upload stream");
    upload_stream.close().await.expect("Can´t close upload stream");
    */
    println!("UPLOAD FILE CHUNK 3 time: {:?}", Local::now().signed_duration_since(start_time));    
    Ok(HttpResponse::Ok().finish())*/
}


#[post("/api/upload/filechunk2")]
async fn upload_file_chunk2(req: HttpRequest,db: web::Data<Arc<MongoServer>>,mut body: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    println!("UPLOAD FILE CHUNK 2");
    let start_time: chrono::DateTime<Local> = Local::now();

    
    let file_name = match get_file_name_from_value(&req) {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File name not found"))),
    };
    let mut upload_stream = db.collections.file_bucket.open_upload_stream(file_name.clone()).await.expect("Can´t open upload stream");



    

    while let Some(chunk) = body.next().await {
        let data = match chunk {
            Ok(data) => data,
            Err(e) => {
                println!("Error reading chunk: {}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };
        upload_stream.write_all(&data).await.expect("Can´t write to upload stream");

    }
    upload_stream.close().await.expect("Can´t close upload stream");

    println!("UPLOAD FILE CHUNK 2 time: {:?}", Local::now().signed_duration_since(start_time));    
    Ok(HttpResponse::Ok().finish())
}




#[post("/api/upload/filechunk")]
async fn upload_file_chunk(req: HttpRequest,db: web::Data<Arc<MongoServer>>, payload: Bytes) -> Result<HttpResponse, actix_web::Error> {
    let start_time: chrono::DateTime<Local> = Local::now();

    let file_token = match get_file_token_from_value(&req) {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File token not found"))),
    };

    let file_name = match get_file_name_from_value(&req) {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File name not found"))),
    };

    let index = match get_file_index_from_value(&req) {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File index not found"))),
    };

    let validate_hash = match get_validate_hash_from_value(&req) {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File validate hash not found"))),
    };

    let dto_filechunk = DTOFileChunk {
        file_name,
        index,
        chunk: payload.clone(),
        validate_hash,
    };
  

    let file_data = match db.get_file_data_from_name(&dto_filechunk.file_name).await {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File not found"))),
    };

    if file_data.validate_file_token(&file_token) == false {
        return Ok(HttpResponse::from(HttpResponse::NotFound().json("File token not found")))
    }

    let file_chunk = match FileChunk::to_file_chunk(dto_filechunk) {
        Some(data) => data,
        None => return Ok(HttpResponse::from(HttpResponse::NotFound().json("File chunk not found"))),
    };
    let file_name = file_chunk.file_name.clone();
    if db.get_file_chunks_by_index(&file_name, &file_chunk.index).await.is_some(){
        return Ok(HttpResponse::from(HttpResponse::NotFound().json("File chunk already exists")));
    }
    if file_data.containts_file_chunk_hash(&file_chunk.validate_hash) == false {
        return Ok(HttpResponse::from(HttpResponse::NotFound().json("File chunk hash not found")));
    }
    println!("Upload File Chunk {:?} before db: {:?} ms", file_data.validate_hash.get_hash(), Local::now().signed_duration_since(start_time));
    if !db.add_file_chunk(&file_chunk).await{
        return Ok(HttpResponse::from(HttpResponse::NotFound().json("Can´t add file chunk")));
    }
    let test: FileData = file_data.clone();
    let size = file_chunk.chunk.len() as u64;
    let file_meta = FileMetadata{ file_name: test.file_name, file_type: file_data.file_type, files_size: size, validate_hash:file_data.validate_hash.clone(), uploader: DiscordID::server()};


    let dbc: web::Data<Arc<MongoServer>> = db.clone();
    tokio::task::spawn(async move {
        dbc.upload_file_to_file_bucket(payload, file_meta).await;
    });




    let response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::of(true)));
    //do something in a new thread and ingore the result
    tokio::task::spawn(async move {
        if db.is_last_file_chunk(&file_name).await {
            println!("LAST CHUNK");
        }
    });
    println!("Upload File Chunk  {:?}  after db: {:?} ms",file_data.validate_hash.get_hash(), Local::now().signed_duration_since(start_time));
    Ok(response)
    
}




pub fn session_error(req: &HttpRequest, user_session:&UserSession, str:&str) -> HttpResponse {
    let mut response = HttpResponse::InternalServerError().json(json!({
        "Error": str
    }));
    set_session_token_cookie(&mut response,  &user_session);
    response
}



#[get("/api/session-data")]
async fn session_data_request(req: HttpRequest, db: web::Data<Arc<MongoServer>>) -> Result<HttpResponse, actix_web::Error> {
    println!("SESSION DATA REQUEST");
    let user_session = get_session_or_create_new(&req, &db).await;
    let sessionrequest : SessionRequest = SessionRequest{
     user_session_id: user_session.user_session_id.clone(), 
     session_token: user_session.session_token.clone() 
    };

    let mut response = HttpResponse::from(HttpResponse::Ok().json(sessionrequest));
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}




pub async fn get_session_with_token_update_or_create_new(req: &HttpRequest, db: &web::Data<Arc<MongoServer>>) -> UserSession {
    let user_session_id = match get_user_id_from_value(&req).or(get_user_id_from_cookie(&req)) {
        Some(data) => data,
        None =>  {
            println!("User-session-id cookie not found");
            return db.new_user_session().await
        }
    };
    let session_token = match get_session_token_from_value(&req).or(get_session_token_from_cookie(&req)) {
        Some(data) => data,
        None =>  {
            println!("Session-cookie not found");
            return db.new_user_session().await
        }
    };
    db.get_user_session_with_token_check(&user_session_id, &session_token).await
}


pub async fn get_session_or_create_new_session_request(session_request:&SessionRequest, db: &web::Data<Arc<MongoServer>>) -> UserSession {
    db.get_user_session_with_token(&session_request.user_session_id, &session_request.session_token).await

}



pub async fn get_session_or_create_new(req: &HttpRequest, db: &web::Data<Arc<MongoServer>>) -> UserSession {
    let user_session_id = match get_user_id_from_value(&req).or(get_user_id_from_cookie(&req)) {
        Some(data) => data,
        None => {
            println!("User-session-id cookie not found");
            return db.new_user_session().await
        }
    };
    let session_token = match get_session_token_from_value(&req).or(get_session_token_from_cookie(&req)) {
        Some(data) => data,
        None => {
            println!("Session-cookie not found");
            return db.new_user_session().await
        }
    };
    db.get_user_session_with_token(&user_session_id, &session_token).await

}



pub async fn find_session(req: &HttpRequest, db: &web::Data<Arc<MongoServer>>) -> Option<UserSession> {
    let user_session_id = match get_user_id_from_value(&req).or(get_user_id_from_cookie(&req)) {
        Some(data) => data,
        None => {
            println!("User-session-id cookie not found");
            return None
        }
    };
    let session_token = match get_session_token_from_value(&req).or(get_session_token_from_cookie(&req)) {
        Some(data) => data,
        None => {
            println!("Session-cookie not found");
            return None
        }
    };
    
    let user = match db.find_user_session(&user_session_id).await {
        Some(data) => data,
        None => {
            println!("User-session not found");
            return None
        }
    };
    if user.session_token.token.eq(&session_token.token) {
        return Some(user);
    }
    None

}


pub fn get_token(req: &HttpRequest) -> Option<usize> {
    let cookie = match req.cookie("token") {
        None => return None,
        Some(cookie) => cookie,
    };
    match cookie.value().parse::<usize>() {
        Err(_) => None,
        Ok(id) => Some(id),
    }
}





pub fn set_cookie(res: &mut HttpResponse, cookie_name: &str, value: &String){
    let cookie = format!("{}={}", cookie_name, value);
    res.headers_mut().append(COOKIE, HeaderValue::from_str(&cookie).unwrap());
    let _cookie = Cookie::build(cookie_name, value)
        .path("/")
        .permanent()
        .secure(true)
        .finish();
    res.add_cookie(&_cookie).expect("Can´t add cookies to the Response");
}


pub fn set_session_token_cookie(response: &mut HttpResponse, user_session: &UserSession){
    set_cookie(response, "user-session-id", &user_session.user_session_id.id.to_string());
    set_cookie(response, "session-token", &user_session.session_token.token);
}






pub fn remove_cookie(res: &mut HttpResponse, req: &HttpRequest, cookie_name: &str){
    if let Some(cookie)= req.cookie(cookie_name) {
        res.add_removal_cookie(&cookie).expect("Can´t add cookies to the Response")
    }
}





#[get("/api/authorization")]
async fn has_authorization(req: HttpRequest, db: web::Data<Arc<MongoServer>>) -> HttpResponse {
    let user_session = get_session_or_create_new(&req, &db).await;
    let admin = is_admin(&user_session, &db).await;
    let mut response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::new(admin)));
    set_session_token_cookie(&mut response, &user_session);
    response
}

#[get("/api/discord_session")]
async fn discord_session(req: HttpRequest, db: web::Data<Arc<MongoServer>>) -> HttpResponse {
    let user_session = get_session_or_create_new(&req, &db).await;
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
    let user_session = get_session_or_create_new(&req, &db).await;
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
    let user_session = get_session_or_create_new(&req, &db).await;


    let lobby_adrr = match srv.send(game::LobbyAddrRequest{lobby_id:lobby_id.clone()}).await.expect("No Lobby found!") {
        Some(data) => data,
        None => return Ok(session_error(&req, &user_session, "No Lobby Found")),
    };



    let can_join = lobby_adrr.send(CanJoinLobby { user_session_id: user_session.user_session_id.clone()}).await.expect("No Lobby found!");

    let mut response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::new(can_join)));
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}

#[get("/api/board")]
async fn board() -> HttpResponse {
    let response = HttpResponse::from(HttpResponse::Ok().json(JeopardyBoard::default(JeopardyMode::SHORT)));
    response
}


pub fn get_user_id_from_cookie(req: &HttpRequest) -> Option<UserSessionId> {
    if let Some(cookie) = req.cookie("user-session-id"){
        if let Ok(id) =  cookie.value().parse::<usize>(){
            return Some(UserSessionId::of(id));
        }
    }
    None
}

pub fn get_session_token_from_cookie(req: &HttpRequest) -> Option<SessionToken> {
    if let Some(cookie) = req.cookie("session-token"){
        return Some(SessionToken {
            token:cookie.value().to_string(),
            expire: Local::now(),
        })
    };
    None
}

pub fn get_user_id_from_value(req: &HttpRequest) -> Option<UserSessionId> {
    if let Ok(cookie) = extract_value(&req,"user-session-id"){
        if let Ok(id) =  cookie.parse::<usize>(){
            return Some(UserSessionId::of(id));
        }
    }
    None
}

pub fn get_session_token_from_value(req: &HttpRequest) -> Option<SessionToken> {
    if let Ok(cookie) = extract_value(&req,"session-token"){
        return Some(SessionToken {
            token:cookie,
            expire: Local::now(),
        })
    };
    None
}

pub fn get_file_token_from_value(req: &HttpRequest) -> Option<DTOFileToken> {
    if let Ok(cookie) = extract_value(&req,"file-token"){
        return Some(DTOFileToken {
            token:cookie,
        })
    };
    None
}

pub fn get_lobby_id_from_value(req: &HttpRequest) -> Option<LobbyId> {
    if let Ok(cookie) = extract_value(&req,"lobby-id"){
        return Some(LobbyId::of(cookie));
    }
    None
}



pub fn get_file_name_from_value(req: &HttpRequest) -> Option<String> {
    if let Ok(cookie) = extract_value(&req,"file-name"){
        return Some(cookie);
    };
    None
}

pub fn get_file_index_from_value(req: &HttpRequest) -> Option<usize> {
    if let Ok(cookie) = extract_value(&req,"file-index"){
        if let Ok(id) =  cookie.parse::<usize>(){
            return Some(id);
        }
    }
    None
}

pub fn get_validate_hash_from_value(req: &HttpRequest) -> Option<ValidateHash> {
    if let Ok(cookie) = extract_value(&req,"validate-hash"){
        return Some(ValidateHash::new(cookie));
    }
    None
}