mod custom_ws;
mod session;
mod server;

use actix::{Actor, ActorFutureExt, Addr, ContextFutureSpawner, fut, StreamHandler, WrapFuture};
use actix_files::NamedFile;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web::{HttpRequest, HttpResponse};
use actix_web_actors::ws;
use futures::task::waker;
use std::any::Any;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Instant;
use actix_web::error::UrlencodedError::ContentType;
use actix_web::http::header::HeaderValue;

use anyhow::{Context, Result};
use futures::FutureExt;
use serde::de::Error;
use crate::session::{PlayerData, WsSession};



#[actix_web::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1";
    let port = 8081;
    let addr = format!("{}:{}", addr, port);
    let addr = addr
        .parse::<SocketAddr>()
        .context("Failed to parse address")?;

    let server = server::GameServer::new().start();


    let server = HttpServer::new(move||
            App::new()
                .app_data(web::Data::new(server.clone()))
                .route("/ws", web::get().to(start_ws))
                .service(ping)
                .service(index)

    )
        .bind(addr)?
        .run();
    println!("Started {} HttpServer! ", addr);
    server.await.expect("Server has crashed!");
    Ok(())
}

async fn start_ws(req: HttpRequest, stream: web::Payload,  srv: web::Data<Addr<server::GameServer>>) -> std::result::Result<HttpResponse, actix_web::Error> {
    let error = "{error: \"Somethings wrong\"}";
    let error_response = Ok(HttpResponse::InternalServerError().json(error));

    //TODO MAKE MATCHES GREAT AGAIN!
    let mut lobby_id = match req.headers().get("lobby-id") {
        None => return error_response,
        Some(value) =>  match value.is_empty() {
            true => return error_response,
            false => value.to_str().expect("String conversion failed!").to_string(),
        }
    };

    let session_token = match req.headers().get("session-token") {
        None => return error_response,
        Some(value) =>  match value.is_empty() {
            true => return error_response,
            false => value.to_str().expect("String conversion failed!").to_string(),
        }
    };

    
    let lobbies = srv.send(server::Rooms).await.expect("No Lobbies found");



    //HACK SET Lobby to main
    //let lobby_id = "main".to_owned();

    if !lobbies.contains(&lobby_id) {
        return error_response
    }

    println!("{} - {}", lobby_id, session_token);
    ws::start(
        WsSession::default(lobby_id, session_token,srv),
        &req,
        stream,)

}

#[get("/amialive")]
async fn ping(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}

#[get("/file/{filename:.*}")]
async fn index(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let mut cexe = env::current_exe().unwrap();
    cexe.pop();
    cexe.push("www");
    cexe.push(path);
    let final_path = cexe.into_os_string().into_string().unwrap();

    println!("path:{}", final_path);
    Ok(NamedFile::open(final_path)?)
}
