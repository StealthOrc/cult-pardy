use std::sync::Arc;
use crate::rest::api::file_part_error;
use crate::rest::error::{ApiError, ApiGameError, ApiRequestError, ApiSessionError, ToApiError, ToResponse};
use crate::data::{SessionRequest};
use crate::services::db::MongoServer;
use crate::services::game::{CreateLobby, FileMetadata, GameServer, GetLobbyMediaToken};
use crate::services::lobby::CanJoinLobby;
use crate::settings::Settings;
use actix::Addr;

use actix_multipart::Multipart;
use actix_web::{get, HttpRequest, HttpResponse, post, web};
use bson::{bson, doc, Bson};
use bytes::Bytes;
use chrono::Local;
use cult_common::dto::api::{ApiResponse};
use cult_common::dto::file::{FileMultiPart};
use cult_common::wasm_lib::hashs::validate::ValidateHash;
use cult_common::wasm_lib::{DiscordUser, JeopardyMode};
use futures::stream::once;
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use mongodb::gridfs::GridFsUploadStream;
use mongodb::options::FindOptions;
use serde::Serialize;
use serde_json::json;
use cult_common::backend::JeopardyBoard;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use utoipa::ToSchema;
use crate::rest::data::{extract_header_string, extract_value, get_internal_server_error_json, get_lobby_id_from_header, get_session, get_session_with_token_update_or_create_new, set_session_token_cookie};
use crate::authentication::discord::is_admin;
use crate::services::game;
use crate::services::game::UserSession;

use super::data::{self, get_media_token_from_header};
use super::error::ApiFileError;

#[utoipa::path(
    post,
    path = "/api/file/upload",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
        ("file-name" = String, Header, description = "File name"),
        ("media-token" = MediaToken, Header, description = "Token do access media file"),
        ("lobby-id" = LobbyId, Header, description = "Lobby ID"),
    ),
    request_body(content = FileMultiPart, description = "Fileupload", content_type = "multipart/form-data"),
    responses(
        // 200
        (status = 200, description = "File uploaded successfully"),


        // Session
        (status = 404, description = "No User Session", body = ApiError),
        (status = 401, description = "Not a admin Session", body = ApiError),
        (status = 403, description = "No Discord Data", body = ApiError),

        //FILE
        (status = 500, description = "File error", body = ApiError),
        (status = 400, description = "File Invalid", body = ApiError),
        (status = 404, description = "File Not Found", body = ApiError),
        (status = 409, description = "File Exists", body = ApiError),

    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[post("/api/file/upload")]
async fn upload_file_part(req: HttpRequest,db: web::Data<Arc<MongoServer>>,mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(ApiSessionError::NotFound.to_api_error().to_response()),
    };
    if is_admin(&user_session, &db).await == false {
        return Ok(file_part_error(None, ApiSessionError::NotAdmin.to_api_error()).await);
    } 
    let discord_id = match user_session.discord_auth.clone() {
        Some(data) => data.discord_user.unwrap().discord_id,
        None => return Ok(file_part_error(None, ApiSessionError::NoDiscordData.to_api_error()).await)};

    let file_name = match extract_header_string(&req, "file-name") {
        Ok(data) => data,
        Err(_) => return Ok(file_part_error(None, ApiFileError::FileInvalid("No file name found".to_string()).to_api_error()).await),
    };



    println!("UPLOAD FILE Part");
    let start_time: chrono::DateTime<Local> = Local::now();
    let mut file_multi = FileMultiPart::default();

    let mut upload_stream = None;

    file_multi.file_name = Some(file_name.clone());

    if let Ok(data) = db.collections.file_bucket_files.find_one(bson::doc!{"filename":Some(file_name.clone())}).await {
        if let Some(_) = data {
            return Ok(file_part_error(None, ApiFileError::FileExists.to_api_error()).await);
        }
    }


    file_multi.uploader_id = Some(discord_id);


    while let Some(item) = payload.next().await {
        match item {
            Ok(mut field) => {
                let name = match field.name() {
                    Some(name) => name.to_owned(),
                    None => return Ok(file_part_error(upload_stream, ApiFileError::FileInvalid("No field name found".to_string()).to_api_error()).await),
                };
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(data) => data,
                        Err(_) => return Ok(file_part_error(upload_stream, ApiFileError::FileInvalid("Can´t get chunk".to_string()).to_api_error()).await),
                    };

                    match name.as_str() {
                        "file_data" => {
                            if upload_stream.is_none() {
                                let file_name = match &file_multi.file_name {
                                    Some(data) => data,
                                    None => return Ok(file_part_error(None, ApiFileError::FileInvalid("Can´t get file name".to_string()).to_api_error()).await),
                                };
                                let file_type = match field.content_type() {
                                    Some(data) => data,
                                    None => return Ok(file_part_error(None, ApiFileError::FileInvalid("Can´t get file type".to_string()).to_api_error()).await),
                                };
                                file_multi.file_type = Some(file_type.to_string()); 
                                let mut stream = db.collections.file_bucket.open_upload_stream(file_name.to_string()).await.expect("Can´t open upload stream");   
                                stream.write_all(&data).await.expect("Can´t write to upload stream");
                                upload_stream = Some(stream);
                            } else {
                                let stream = match &mut upload_stream {
                                    Some(data) => data,
                                    None => return Ok(file_part_error(None, ApiFileError::FileInvalid("Can´t get upload stream".to_string()).to_api_error()).await),                       
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
            Err(_) => return Ok(file_part_error(upload_stream, ApiFileError::FileInvalid("Can´t get field".to_string()).to_api_error()).await),
        }
    }

    println!("UPLOAD FILE PART size: {:?} in {:?}", file_multi.file_name, Local::now().signed_duration_since(start_time));
    println!("{:#?}", file_multi);
    println!("{:#?}", upload_stream.is_none());

    if !file_multi.is_valid() {
        return Ok(file_part_error(upload_stream, ApiFileError::FileInvalid("File is not valid".to_string()).to_api_error()).await);
    }

    if let Some(mut stream) = upload_stream {
        let id = stream.id().clone();

        if let Ok(data) = db.collections.file_bucket_files.find_one(bson::doc!{"filename":Some(file_name.clone())}).await {
            if let Some(_) = data {
                return Ok(file_part_error(Some(stream), ApiFileError::FileExists.to_api_error()).await);
            }
        }
        if let Err(_) = stream.flush().await {
            return Ok(file_part_error(Some(stream), ApiFileError::FileError("Can´t flush upload stream".to_string()).to_api_error()).await);
        }
        if let Err(_) = stream.close().await {
            return Ok(file_part_error(Some(stream), ApiFileError::FileError("Can´t close upload stream".to_string()).to_api_error()).await);
        }

        let file_meta = FileMetadata{
            file_type: file_multi.file_type.unwrap(),
            validate_hash: file_multi.validate_hash.unwrap(),
            uploader: file_multi.uploader_id.unwrap(),
        };
        println!("{:#?}", file_meta);

        if let Err(_) = db.collections.file_bucket_files.update_one(bson::doc! { "_id":id}, bson::doc! {"$set": {"metadata": file_meta}}).await {
            return Ok(file_part_error(Some(stream), ApiFileError::FileError("Can´t update file metadata".to_string()).to_api_error()).await);
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



#[get("/api/file/download")]
async fn get_file_from_name(req: HttpRequest,  db: web::Data<Arc<MongoServer>>, settings:web::Data<Arc<Settings>>, game_server:web::Data<Addr<GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(file_part_error(None, ApiSessionError::NotFound.to_api_error()).await),
    };

    let file_name = match extract_header_string(&req, "file-name") {
        Ok(data) => data,
        Err(e) => return Ok(e),
    };


    let media_token = match get_media_token_from_header(&req) {
        Some(data) => data,
        None => return Ok(file_part_error(None, ApiFileError::FileInvalid("No media token found".to_string()).to_api_error()).await),
    }; 

    let lobby_id = match extract_header_string(&req, "lobby-id") {
        Ok(data) => LobbyId::of(data),
        Err(e) => return Ok(e)
    };


    let opt_token =  match game_server.send(GetLobbyMediaToken{
        lobby_id: lobby_id.clone()
    }).await {
        Ok(data) => data,
        Err(_) => return Ok(file_part_error(None, ApiFileError::FileError("Can´t get token".to_string()).to_api_error()).await),
    };

    let token = match opt_token {
        Some(data) => data,
        None => return Ok(file_part_error(None, ApiFileError::FileError("No token found".to_string()).to_api_error()).await),
    };


    println!("{:#?} == {:#?}", token, media_token);

    if token != media_token {
        return Ok(file_part_error(None, ApiFileError::FileError("Token not valid".to_string()).to_api_error()).await);
    
    
    
    }

        
    let file_data = match db.collections.file_bucket_files.find_one(doc!{"filename":Some(file_name.clone())}).await {
        Ok(data) => {
            if let Some(data) = data {
                  data
            } else {
                return Ok(file_part_error(None, ApiFileError::FileNotFound(file_name).to_api_error()).await);
            }
            
        }
        Err(err) => {
            println!("Error while downloading file: {:#?}", err);
            return Ok(file_part_error(None, ApiFileError::FileError("Error while downloading file".to_string()).to_api_error()).await);
        }
    };

    let mut test = match db.collections.file_bucket.open_download_stream_by_name(file_name.clone()).await {
        Ok(data) => data,
        Err(_) => return Ok(file_part_error(None, ApiFileError::FileError("Error while downloading file1".to_string()).to_api_error()).await),
    };
    println!("Downloading file: {}", file_name);

    let file_meta = match file_data.metadata{
        Some(data) => data,
        None => return Ok(file_part_error(None, ApiFileError::FileError("Error while downloading file2".to_string()).to_api_error()).await),
    };



    let mut buf = Vec::new();
    if let Err(_) = test.read_to_end(&mut buf).await {
        return Ok(file_part_error(None, ApiFileError::FileError("Error while downloading file3".to_string()).to_api_error()).await);
    }
    
    let mut response =  HttpResponse::Ok()
                                        .insert_header(("file-name", file_name))
                                        .insert_header(("file-type", file_meta.file_type))
                                        .insert_header(("file-size", file_data.length))
                                        .insert_header(("file-upload-date", file_data.upload_date.to_string()))
                                        .insert_header(("uploader-id", file_meta.uploader.id))
                                        .insert_header(("validate-hash", file_meta.validate_hash.get_hash()))
                                        .content_type("application/octet-stream")
                                        .streaming(once(async move {
                                            Ok::<_, actix_web::Error>(Bytes::from(buf))
                                        }));

    set_session_token_cookie(&mut response, &settings,&user_session);
    Ok(response)
}



#[utoipa::path(
    get,
    path = "/api/files",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
    ),
    responses(
        // 200
        (status = 200, description = "FIX ME"),
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[get("/api/files")]
async fn get_file_size(req: HttpRequest,  db: web::Data<Arc<MongoServer>>, settings:web::Data<Arc<Settings>>) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(file_part_error(None, ApiSessionError::NotFound.to_api_error()).await),
    };

    if is_admin(&user_session, &db).await == false {
        return Ok(file_part_error(None, ApiSessionError::NotAdmin.to_api_error()).await);
    }

    let file_count = match db.collections.file_bucket_files.count_documents(doc!{}).await {
        Ok(data) => data,
        Err(_) => 0,
    };


    let mut response = HttpResponse::Ok().json(json!({
        "file_count": file_count,
    }));

    set_session_token_cookie(&mut response, &settings,&user_session);
    Ok(response)


}


#[utoipa::path(
    get,
    path = "/api/file/list",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
        ("page_size" = u64, Header, description = "Token do access media file"),
        ("page" = u64, Header, description = "page"),
    ),
    responses(
        // 200
        (status = 200, description = "FIX ME"),
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[get("/api/file/list")]
async fn get_file_list(req: HttpRequest,  db: web::Data<Arc<MongoServer>>, settings:web::Data<Arc<Settings>>) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(file_part_error(None, ApiSessionError::NotFound.to_api_error()).await),
    };
    if is_admin(&user_session, &db).await == false {
        return Ok(file_part_error(None, ApiSessionError::NotAdmin.to_api_error()).await);
    }

    let page = match extract_header_string(&req, "page") {
        Ok(data) => u64::from_str_radix(&data, 10).unwrap_or(0),
        Err(e) => return Ok(e),
    };

    let page_size = match extract_header_string(&req, "page_size") {
        Ok(data) => u64::from_str_radix(&data, 10).unwrap_or(10),
        Err(e) => return Ok(e),
    };

    // Count total files
    let file_count = match db.collections.file_bucket_files.count_documents(doc! {}).await {
        Ok(count) => count,
        Err(_) => return Ok(file_part_error(None, ApiFileError::FileError("Error counting files".to_string()).to_api_error()).await),
    };

    // Validate pagination
    if page_size == 0 {
        return Ok(file_part_error(None, ApiFileError::FileError("Page size cannot be zero".to_string()).to_api_error()).await);
    }
    
    if page > file_count / page_size {
        return Ok(file_part_error(None, ApiFileError::FileError("Page out of range".to_string()).to_api_error()).await);
    }

    // Prepare MongoDB find options
    let skip = page * page_size as u64;
    let find_options = FindOptions::builder()
        .sort(doc! { "filename": 1 })  // Sort by filename in ascending order
        .skip(skip)
        .limit(page_size as i64)
        .build();



    let mut cursor  = match db.collections.file_bucket_files.find(doc!{}).with_options(find_options).await {
        Ok(data) => data,
        Err(_) => return Ok(file_part_error(None, ApiFileError::FileError("Error getting files".to_string()).to_api_error()).await),
    };

    let mut dto_file_data = Vec::new();

    while let Some(result) = cursor.next().await {
        match result {
            Ok(data) => {
                match data.to_dto() {
                    Some(data) => dto_file_data.push(data),
                    None => return Ok(file_part_error(None, ApiFileError::FileError("Error converting file data".to_string()).to_api_error()).await),
                }
            },
            Err(_) => return Ok(file_part_error(None, ApiFileError::FileError("Error getting file data".to_string()).to_api_error()).await),
        }
        
    }

    let mut response = HttpResponse::Ok().json(json!({
        "files": dto_file_data,
        "page": page,
        "page_size": page_size,
    }));




    set_session_token_cookie(&mut response, &settings,&user_session);
    Ok(response)
}
