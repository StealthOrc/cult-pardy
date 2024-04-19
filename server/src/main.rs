mod custom_ws;

use std::any::Any;
use std::net::SocketAddr;
use std::path::PathBuf;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_files::NamedFile;
use actix_web::{
    HttpRequest, HttpResponse,
};
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;

use anyhow::{Context, Result};
use serde::de::Error;


#[actix_web::main]
async fn main() -> Result<()>{
    let addr = "127.0.0.1";
    let port = 8080;
    let addr = format!("{}:{}", addr, port);
    let addr = addr.parse::<SocketAddr>()
        .context("Failed to parse address")?;

    let server = HttpServer::new(|| {
        App::new()
            .service(start_ws)
            .service(ping)
            .service(index)
    })
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
    let mut wwwdir: String = String::from("\\www\\");
    wwwdir.push_str(path.to_str().unwrap());
    println!("{wwwdir}");
    Ok(NamedFile::open(wwwdir)?)
}