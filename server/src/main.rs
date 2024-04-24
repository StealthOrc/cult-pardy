mod api;
mod gamewebsocket;
mod error;
mod lib;
mod server;
mod session;

use actix::Actor;

use actix_web::{web, App, HttpServer};
use anyhow::Result;

use cult_common::*;

#[actix_web::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1";
    let port = 8000;
    let addr = parse_addr_str(addr, port);

    let server = server::GameServer::new().start();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .route("/ws", web::get().to(gamewebsocket::start_ws))
            .service(api::game_info)
    })
    .bind(addr)?
    .run();
    println!("Started {} HttpServer! ", addr);
    server.await.expect("Server has crashed!");
    Ok(())
}
