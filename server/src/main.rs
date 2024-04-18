use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use cult_common::hello_world;

use actix_files::NamedFile;
use actix_web::HttpRequest;
use std::path::PathBuf;

async fn index(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let mut wwwdir: String = String::from("\\www\\");
    wwwdir.push_str(path.to_str().unwrap());
    println!("{wwwdir}");
    Ok(NamedFile::open(wwwdir)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{web, App, HttpServer};

    HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
