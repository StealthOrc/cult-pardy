mod apis;
mod frontend;
mod ws;
mod authentication;
mod servers;

use std::any::Any;
use std::sync::Arc;
use actix::{Actor};

use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, post, get};
use anyhow::Result;
use tokio::io;
use tokio::runtime::Runtime;
use cult_common::*;
use cult_common::JeopardyMode::NORMAL;
use crate::apis::api::{create_game_lobby,  has_authorization, session};
use crate::authentication::auth;
use crate::authentication::auth::{GrantDiscordAuth, LoginDiscordAuth};
use crate::frontend::frontend::{assets, find_game, grant_admin_access, index, test};
use crate::servers::authentication::AuthenticationServer;
use crate::servers::input::{InputServer};
use crate::servers::game::{GameServer};
use crate::servers::Services;
use crate::ws::gamewebsocket;

#[actix_web::main]
async fn main() -> Result<()> {

    let addr = "127.0.0.1";
    let port = 8000;
    let addr = parse_addr_str(addr, port);

    let services = Services::init();





    let input_server =  InputServer::init(services.authentication_server.clone());
    let rt = Runtime::new().expect("Somethings wrong with the Runtime");
    rt.spawn(input_server.read_input());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(services.grant_client.clone()))
            .app_data(web::Data::new(services.login_client.clone()))
            .app_data(web::Data::new(services.game_server.clone()))
            .app_data(web::Data::new(services.authentication_server.clone()))
            .route("/ws", web::get().to(gamewebsocket::start_ws))
            .service(auth::discord_oauth)
            .service(auth::grant_access)
            .service(auth::login_only)
            .service(index)
            .service(find_game)
            .service(assets)
            .service(test)
            .service(session)
            .service(grant_admin_access)
            .service(has_authorization)
            .service(download)
    })
    .bind(addr)?
    .run();
    println!("Started {} HttpServer! ", addr);
    server.await.expect("Server has crashed!");
    Ok(())
}

#[get("/api/download")]
async fn download(req: HttpRequest) -> std::result::Result<HttpResponse, actix_web::Error> {
    let json_data = serde_json::to_string_pretty(&JeopardyBoard::default(NORMAL)).expect("Test?");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .append_header(("Content-Disposition", "attachment; filename=test.json"))
        .body(json_data))
}