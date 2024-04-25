
use std::env;
use std::path::PathBuf;
use actix::Addr;

use actix_files::NamedFile;
use actix_web::{get, HttpRequest, HttpResponse, patch, post, web};
use actix_web::web::Json;
use serde_json::json;
use cult_common::{UserSessionRequest};
use crate::data::{extract_header_string, extract_value};
use crate::server;
use crate::server::UserSession;

#[get("/api/info")]
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


#[post("/api/session")]
async fn session(req: HttpRequest, session_request: Option<web::Json<UserSessionRequest>>, srv: web::Data<Addr<server::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    let session = match session_request {
        None =>  srv.send(server::UserSession{user_session_request: None}),
        Some(json) => {
            println!("{:?}", json.0);
            srv.send(server::UserSession{user_session_request:Some(json.0)})
        },
    }.await.expect("No User Session can be created");
    Ok(HttpResponse::from(HttpResponse::Ok().json(UserSessionRequest{session_id:session})))
}



#[get("/api/authorization")]
async fn has_authorization(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}

#[post("/api/create")]
async fn create_game_lobby(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}


#[post("/api/join")]
async fn join_game(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}


#[patch("/api/update-authorization")]
async fn update_authorization(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}


