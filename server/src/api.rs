
use std::env;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use actix::Addr;

use actix_files::NamedFile;
use actix_web::{get, HttpRequest, HttpResponse, patch, post, web};
use actix_web::cookie::{Cookie, Expiration};
use actix_web::cookie::time::OffsetDateTime;
use actix_web::web::{get, Json};
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
async fn session(session_request: Option<web::Json<UserSessionRequest>>, srv: web::Data<Addr<server::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match session_request {
        None => server::UserSession { user_session_request: None },
        Some(json) => server::UserSession { user_session_request: Some(json.0) },
    };
    let session = srv.send(user_session).await.expect("No User Session Found");
    Ok(HttpResponse::from(HttpResponse::Ok().json(UserSessionRequest{session_id:session})))
}



pub async fn get_session(req: &HttpRequest, srv: &web::Data<Addr<server::GameServer>>) -> usize {
    let user_req = match req.cookie("user_session_id") {
        None => UserSessionRequest::default(),
        Some(cookie) => match cookie.value().parse::<usize>() {
            Err(_) => UserSessionRequest::default(),
            Ok(id) => UserSessionRequest{session_id: id},
        }
    };
    srv.send(server::UserSession{user_session_request:Some(user_req)}).await.expect("Somethings wrong with sessions")
}



pub fn set_session_cookies(res: &mut HttpResponse, cookie_name: &str, cookie: &str){
   //let expiration_time = SystemTime::now() + Duration::from_secs(60);
    let cookie = Cookie::build(cookie_name, cookie)
        .path("/")
        .secure(true)
        //TODO Do we need this?
        //.expires(Expiration::DateTime(OffsetDateTime::from(expiration_time)))
        .finish();
    res.add_cookie(&cookie).expect("Can´t add cookies to the Response");
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


