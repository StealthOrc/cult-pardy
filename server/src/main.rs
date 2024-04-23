mod session;
mod server;
mod api;
mod ws;
mod error;

use actix::{Actor, ActorFutureExt, Addr, ContextFutureSpawner, StreamHandler, WrapFuture};
use actix_web::{web, App, HttpServer, Responder, get, HttpRequest, HttpResponse, post};
use std::any::Any;
use std::net::SocketAddr;
use anyhow::{Context, Result};
use futures::FutureExt;
use serde::de::Error;
use cult_common::extract_header_string;
use crate::api::{index};

#[actix_web::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1";
    let port = 8000;
    let addr = format!("{}:{}", addr, port);
    let addr = addr
        .parse::<SocketAddr>()
        .context("Failed to parse address")?;

    let server = server::GameServer::new().start();


    let server = HttpServer::new(move||
            App::new()
                .app_data(web::Data::new(server.clone()))
                .route("/ws", web::get().to(ws::start_ws))
                .service(api::game_info)

    )
        .bind(addr)?
        .run();
    println!("Started {} HttpServer! ", addr);
    server.await.expect("Server has crashed!");
    Ok(())
}
