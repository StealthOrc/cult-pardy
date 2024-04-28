#![feature(async_closure)]
mod apis;
mod frontend;
mod ws;
mod authentication;
mod servers;

use std::any::Any;
use std::sync::Arc;
use actix::{Actor};

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use tokio::io;
use tokio::runtime::Runtime;
use cult_common::*;
use crate::apis::api::{game_info, session};
use crate::authentication::auth;
use crate::authentication::auth::callback;
use crate::frontend::frontend::{assets, find_game, grant_admin_access, index, test};
use crate::servers::authentication::AuthenticationServer;
use crate::servers::input::{InputServer};
use crate::servers::game::GameServer;
use crate::ws::gamewebsocket;

#[actix_web::main]
async fn main() -> Result<()> {

    let addr = "127.0.0.1";
    let port = 8000;
    let addr = parse_addr_str(addr, port);
    let oauth_client = auth::create_oauth_client();

    let auth_server = AuthenticationServer::new().start();
    let game_server = GameServer::new().start();

    let input_server =  InputServer::init(auth_server.clone());
    let rt = Runtime::new().expect("Somethings wrong with the Runtime");
    rt.spawn(input_server.read_input());



    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(oauth_client.clone()))
            .app_data(web::Data::new(game_server.clone()))
            .app_data(web::Data::new(auth_server.clone()))
            .route("/ws", web::get().to(gamewebsocket::start_ws))
            .route("/discord", web::get().to(auth::discord_oauth))
            .route("/callback", web::get().to(callback))
            .service(game_info)
            .service(index)
            .service(assets)
            .service(test)
            .service(session)
            .service(find_game)
            .service(grant_admin_access)
    })
    .bind(addr)?
    .run();
    println!("Started {} HttpServer! ", addr);
    server.await.expect("Server has crashed!");
    Ok(())
}





