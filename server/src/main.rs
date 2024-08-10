mod apis;
mod frontend;
mod ws;
mod authentication;
mod servers;
mod data;


use std::sync::Arc;

use crate::apis::api::{board, create_game_lobby, discord_session, has_authorization, join_game};
use crate::apis::api::api_session_request;

use actix::Addr;
use actix_web::error::ErrorBadRequest;
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::Message;
use anyhow::Result;

use apis::api::{ get_session_or_create_new, session_data_request, set_session_token_cookie, upload_file_chunk, upload_file_chunk2, upload_file_chunk3, upload_file_data};
use apis::data::extract_header_string;
use authentication::discord::is_admin;
use bson::binary;
use bytes::BytesMut;
use dto::{file, DTOFileData};
use futures::{AsyncReadExt, StreamExt};
use servers::authentication::AuthenticationServer;
use servers::db::{self, MongoServer};
use servers::game::GameServer;
use tokio::runtime::Runtime;
use cult_common::*;
use cult_common::backend::JeopardyBoard;
use wasm_lib::JeopardyMode;
use ws::filewebsocket;
use crate::authentication::discord;
use crate::frontend::frontend::{assets, find_game, grant_admin_access, index};
use crate::servers::input::{InputServer};
use crate::servers::Services;
use crate::ws::gamewebsocket;






#[actix_web::main]
async fn main() -> Result<()> {

    let addr = "0.0.0.0";
    let port = 8000;
    let addr = parse_addr_str(addr, port);

    let services = Services::init().await;

    std::env::set_var("RUST_LOG", "debug");
    //env_logger::init();


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
            .route("/filews", web::get().to(filewebsocket::start_ws))
            .route("/filews2", web::get().to(ws))
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
            .service(download)
            .service(upload_file_chunk)
            .service(upload_file_data)
            .service(get_file_from_name)
            .service(get_file_from_name2)
            .service(upload_streaming_data)
            .service(upload_file_chunk2)
            .service(upload_file_chunk3)
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

async fn not_found() -> std::result::Result<HttpResponse, actix_web::Error> {
    let response = HttpResponse::PermanentRedirect()
        .append_header(("Location", format!("{}{}",PROTOCOL,LOCATION)))
        .finish();
    Ok(response)
}



#[get("/api/download")]
async fn download(_req: HttpRequest) -> std::result::Result<HttpResponse, actix_web::Error> {
    let json_data = serde_json::to_string_pretty(&JeopardyBoard::default(JeopardyMode::NORMAL)).expect("Test?");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .append_header(("Content-Disposition", "attachment; filename=test.json"))
        .body(json_data))
}

#[post("/api/upload")]
async fn upload_streaming_data(mut payload: web::Payload) -> std::result::Result<HttpResponse, actix_web::Error> {

    let bytes = &mut Vec::new();

    while let Some(chunk) = payload.next().await {
        let result =  match chunk {
            Ok(data) => data,
            Err(_) => return Ok(HttpResponse::BadRequest().finish()),
        };
        bytes.extend_from_slice(&result);

    }
    println!("Received {} bytes", bytes.len());

    Ok(HttpResponse::Ok().finish())
}



#[get("/api/file/{name}")]
async fn get_file_from_name(path: web::Path<String>, req: HttpRequest,  db: web::Data<Arc<MongoServer>> , auth: web::Data<Addr<AuthenticationServer>> ) -> Result<HttpResponse, actix_web::Error> {
    let user_session = get_session_or_create_new(&req, &db).await;

    let name = path.into_inner();
    if name.is_empty() {
        return Ok(HttpResponse::from(HttpResponse::BadRequest()));
    }
        
    if is_admin(&user_session, &db).await == false {
        return Ok(HttpResponse::from(HttpResponse::Unauthorized()));
    }

    let file = db.get_cfile_from_name(&name).await;



    let mut response = match file {
        None => HttpResponse::from(HttpResponse::NotFound()),
        Some(data) => {
            let bytes = data.get_as_bytes();
            HttpResponse::Ok()
            .insert_header(("file-name", data.file_data.file_name))
            .insert_header(("validate-hash", data.file_data.validate_hash.get_hash()))
            .insert_header(("file-type", data.file_data.file_type))
            .insert_header(("uploader", data.file_data.uploader.id))
            .insert_header(("upload-data", data.file_data.upload_data.to_string()))
            .content_type("application/octet-stream")
            .body(bytes)
        }
    };
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}



#[get("/api/file2/{name}")]
async fn get_file_from_name2(path: web::Path<String>, req: HttpRequest,  db: web::Data<Arc<MongoServer>> , auth: web::Data<Addr<AuthenticationServer>> ) -> Result<HttpResponse, actix_web::Error> {
    let user_session = get_session_or_create_new(&req, &db).await;

    let name = path.into_inner();
    if name.is_empty() {
        return Ok(HttpResponse::from(HttpResponse::BadRequest()));
    }
        
    if is_admin(&user_session, &db).await == false {
        return Ok(HttpResponse::from(HttpResponse::Unauthorized()));
    }

    //let file = db.get_cfile_from_name(&name).await;

    let mut test = db.collections.file_bucket.open_download_stream_by_name(&name).await.expect("Failed to open download stream");
    let mut buf = Vec::new();
    let result = test.read_to_end(&mut buf).await?;

    
    

    let mut response =             HttpResponse::Ok()
    .content_type("application/octet-stream")
    .body(buf);



    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}



async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    println!("Starting WS2");
    let mut file_chunks = BytesMut::new();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            let mut session2 = session.clone();
            match msg {
                Message::Ping(bytes) => {
                    session2.pong(&bytes).await.expect("Failed to send pong");


                }
                Message::Text(msg) => println!("Got text: {msg}"),
                Message::Binary(data) => {
                    file_chunks.extend_from_slice(&data);
                    if session2.text("ack").await.is_err() {
                        return;
                    }
                }
                Message::Continuation(_) => todo!(),
                Message::Pong(_) => {
                    session2.ping(b"").await.expect("Failed to send ping");
                }
                Message::Close(_) => {
                    println!("Recived {} bytes", file_chunks.len());
                    session2.close(None).await.expect("Failed to close connection");
                    return;
                }
                Message::Nop =>{
                    println!("Nop");
                    session2.close(None).await.expect("Failed to close connection");
                }
            }
        }
    });

    Ok(response)
}