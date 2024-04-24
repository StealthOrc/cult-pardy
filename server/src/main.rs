mod api;
mod gamewebsocket;
mod error;
mod server;
mod session;
mod ws;

use crate::api::index;
use actix::{Actor, ActorFutureExt, Addr, ContextFutureSpawner, StreamHandler, WrapFuture};
use actix_files::NamedFile;
use actix_web::error::UrlencodedError::ContentType;
use actix_web::http::header::HeaderValue;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::{Context, Result};
use cult_common::extract_header_string;
use cult_common::*;
use futures::task::waker;
use futures::FutureExt;
use serde::de::Error;
use std::path::PathBuf;
use std::time::Instant;

use crate::session::{PlayerData, WsSession};

#[actix_web::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1";
    let port = 8000;
    let addr = parse_addr_str(addr, port);

    let server = server::GameServer::new().start();


    let server = HttpServer::new(move||
            App::new()
                .app_data(web::Data::new(server.clone()))
                .route("/ws", web::get().to(gamewebsocket::start_ws))
                .service(api::game_info)

    )
        .bind(addr)?
        .run();
    println!("Started {} HttpServer! ", addr);
    server.await.expect("Server has crashed!");
    Ok(())
}
