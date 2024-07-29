mod apis;
mod frontend;
mod ws;
mod authentication;
mod servers;


use crate::apis::api::{board, create_game_lobby, discord_session, has_authorization, join_game};
use crate::apis::api::session_request;

use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, get};
use anyhow::Result;

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



    let input_server =  InputServer::init(services.authentication_server.clone());
    let rt = Runtime::new().expect("Somethings wrong with the Runtime");
    rt.spawn(input_server.read_input());


    let server = HttpServer::new(move || {
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
            .service(session_request)
            .service(grant_admin_access)
            .service(has_authorization)
            .service(board)
            .service(discord_session)
            .service(create_game_lobby)
            .service(join_game)
            .service(download)
            .default_service(
                web::route().to(not_found)
            )
    })
    .bind(addr)?
    .run();
    println!("Started {} HttpServer! ", addr);
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