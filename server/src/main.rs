mod custom_ws;

use actix::{Actor, StreamHandler};
use actix_files::NamedFile;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web::{HttpRequest, HttpResponse};
use actix_web_actors::ws;
use futures::task::waker;
use std::any::Any;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::de::Error;

#[actix_web::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1";
    let port = 8081;
    let addr = format!("{}:{}", addr, port);
    let addr = addr
        .parse::<SocketAddr>()
        .context("Failed to parse address")?;

    let server = HttpServer::new(|| App::new().service(start_ws).service(ping).service(index))
        .bind(addr)?
        .run();
    println!("Started {} HttpServer! ", addr);
    server.await.expect("Server has crashed!");
    Ok(())
}

#[get("/ws")]
async fn start_ws(req: HttpRequest, stream: web::Payload) -> HttpResponse {
    ws::start(custom_ws::GameWS, &req, stream).expect("Can´t start WS")
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
