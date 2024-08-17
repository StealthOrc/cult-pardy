mod apis;
mod frontend;
mod ws;
mod authentication;
mod services;
mod data;
mod settings;


use std::sync::Arc;

use crate::apis::api::{board, create_game_lobby, discord_session, has_authorization, join_game};
use crate::apis::api::api_session_request;

use actix_web::error::ErrorBadRequest;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::Result;

use apis::api::{session_data_request, upload_file_part};
use apis::data::{extract_header_string, get_session, set_session_token_cookie};
use authentication::discord::is_admin;
use bson::doc;
use bytes::Bytes;
use futures::stream::once;
use futures::AsyncReadExt;
use services::db::MongoServer;
use settings::Settings;
use tokio::runtime::Runtime;
use cult_common::*;
use crate::authentication::discord;
use crate::frontend::frontend::{assets, find_game, grant_admin_access, index};
use crate::services::input::InputServer;
use crate::services::Services;
use crate::ws::gamewebsocket;







#[actix_web::main]
async fn main() -> Result<()> {
    let settings = Arc::new(Settings::new().expect("Failed to load configuration"));
    

    let addr = settings.backend_settings.host.as_str();
    let port = settings.backend_settings.port;
    let addr = parse_addr_str(addr, port);

    let services = Services::init(&settings).await;

    //std::env::set_var("RUST_LOG", "debug");

    let input_server =  InputServer::init(services.authentication_server.clone());
    let rt = Runtime::new().expect("Somethings wrong with the Runtime");

    rt.spawn(input_server.read_input());
    println!("Workers available: {}", num_cpus::get());
    println!("Starting HTTP server at {}", addr); 

    let server = 
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(services.mongo_server.clone()))
            .app_data(web::Data::new(services.grant_client.clone()))
            .app_data(web::Data::new(services.login_client.clone()))
            .app_data(web::Data::new(services.game_server.clone()))
            .app_data(web::Data::new(services.authentication_server.clone()))
            .app_data(web::Data::new(settings.clone()))
            .app_data(
                web::JsonConfig::default()
                    .limit(104857600) // Increase JSON JsonConfig limit (100MB)
                    .error_handler(|err, _req| {
                        let error_message = format!("Error: {}", err);
                        ErrorBadRequest(error_message)
                    })
            )
            .app_data(
                web::FormConfig::default()
                    .limit(104857600) // Increase FormConfig limit (100MB)
                    .error_handler(|err, _req| {
                        let error_message = format!("Error: {}", err);
                        ErrorBadRequest(error_message)
                    })
            )
            .app_data(web::PayloadConfig::default() .limit(104857600)) // Increase PayloadConfig limit (100MB)
            .route("/ws", web::get().to(gamewebsocket::start_ws))
            .service(discord::discord_oauth)
            .service(discord::grant_access)
            .service(discord::login_only)
            .service(index)
            .service(find_game)
            .service(assets)
            .service(api_session_request)
            .service(session_data_request)
            .service(grant_admin_access)
            .service(has_authorization)
            .service(board)
            .service(discord_session)
            .service(create_game_lobby)
            .service(join_game)
            .service(get_file_from_name)
            .service(upload_file_part)
            .default_service(
                web::route().to(not_found)
            )
    })
    .bind(addr)?
    .workers(num_cpus::get()/2)
    .run();

    server.await?;
    rt.shutdown_background();
    Ok(())
}

async fn not_found(settings:web::Data<Arc<Settings>>) -> std::result::Result<HttpResponse, actix_web::Error> {
    let url = format!("{}", settings.backend_settings.get_host());
    let response = HttpResponse::PermanentRedirect()
        .append_header(("Location", url))
        .finish();
    Ok(response)
}




pub struct CFile {
    pub file_name: String,
    pub file_type: String,
    pub data: Bytes,
}




#[get("/api/file")]
async fn get_file_from_name(req: HttpRequest,  db: web::Data<Arc<MongoServer>>) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(HttpResponse::Unauthorized().json("You are not authorized to access this file")),
    };

    let file_name = match extract_header_string(&req, "file-name") {
        Ok(data) => data,
        Err(_) => return Ok(HttpResponse::BadRequest().json("No file name provided")),
    };
        
    if is_admin(&user_session, &db).await == false {
        return Ok(HttpResponse::from(HttpResponse::Unauthorized().json("You are not authorized to access this file")));
    }
    let file_data = match db.collections.file_bucket_files.find_one(doc!{"filename":Some(file_name.clone())}).await {
        Ok(data) => {
            if let Some(data) = data {
                  data
            } else {
                return Ok(HttpResponse::NotFound().json("File not found"));
            }
            
        }
        Err(err) => {
            println!("Error while downloading file: {:#?}", err);
            return Ok(HttpResponse::InternalServerError().json("Error while downloading file1"));
        }
    };

    let mut test = match db.collections.file_bucket.open_download_stream_by_name(file_name.clone()).await {
        Ok(data) => data,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Error while downloading file")),
    };
    println!("Downloading file: {}", file_name);

    let file_meta = match file_data.metadata{
        Some(data) => data,
        None => return Ok(HttpResponse::InternalServerError().json("Error while downloading file2")),
    };



    let mut buf = Vec::new();
    if let Err(_) = test.read_to_end(&mut buf).await {
        return Ok(HttpResponse::InternalServerError().json("Error while downloading file3"));
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

    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}



    