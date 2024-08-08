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
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::Result;

use apis::api::{ get_session_or_create_new, session_data_request, set_session_token_cookie, upload_file_chunk, upload_file_data};
use apis::data::extract_header_string;
use authentication::discord::is_admin;
use dto::DTOFileData;
use futures::StreamExt;
use servers::authentication::AuthenticationServer;
use servers::db::{self, MongoServer};
use servers::game::GameServer;
use tokio::runtime::Runtime;
use cult_common::*;
use cult_common::backend::JeopardyBoard;
use wasm_lib::JeopardyMode;
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

    println!("Starting HTTP server at {}", addr); 
    let server = 
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(services.mongo_server.clone()))
            .app_data(web::Data::new(services.grant_client.clone()))
            .app_data(web::Data::new(services.login_client.clone()))
            .app_data(web::Data::new(services.game_server.clone()))
            .app_data(web::Data::new(services.authentication_server.clone()))
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
            .service(download)
            .service(upload_file_chunk)
            .service(upload_file_data)
            .service(get_file_from_name)
            .service(upload_streaming_data)
            .default_service(
                web::route().to(not_found)
            )
    })
    .bind(addr)?
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
        Some(data) => HttpResponse::from(HttpResponse::Ok().json(data.to_dto())),
    };

    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}
