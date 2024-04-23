use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use actix::Addr;
use actix::dev::Request;
use actix_files::NamedFile;
use actix_web::{get, HttpRequest, HttpResponse, patch, post, web};
use serde_json::json;
use cult_common::{extract_header_string, extract_value};
use crate::server;

#[get("/info")]
async fn game_info(req: HttpRequest, srv: web::Data<Addr<server::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    println!("{:?}", extract_value(&req, "key"));

    let lobby_id = match extract_header_string(&req, "lobby-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };
    let lobby = srv.send(server::Lobby{lobby_id: lobby_id.clone()}).await.expect("No Lobby found!");
    let error = json!(
        {
            "Error": "Lobby not found",
            "Lobby": lobby_id
        }
    );
    let user =match lobby {
        None => return Ok(HttpResponse::from(HttpResponse::InternalServerError().json(error))),
        Some(users) => users
    };
    Ok(HttpResponse::from(HttpResponse::Ok().json(user)))
}

#[get("/authorization")]
async fn has_authorization(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}

#[post("/create")]
async fn create_game_lobby(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}


#[post("/join")]
async fn join_game(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}


#[patch("/update-authorization")]
async fn update_authorization(_req: HttpRequest) -> HttpResponse {
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
