use std::time::{Duration, Instant};
use crate::servers::game::{GetUserSession, LobbyId};
use actix::{Addr, MailboxError};

use actix_web::cookie::{Cookie};
use actix_web::{get, HttpRequest, HttpResponse, patch, post, web};
use chrono::Local;
use serde::Serialize;
use serde_json::json;
use cult_common::JeopardyMode::NORMAL;
use cult_common::{JeopardyBoard, SessionToken, UserSessionId};
use crate::apis::data::{extract_header_string, extract_value};
use crate::authentication::discord::is_admin;
use crate::servers::authentication::AuthenticationServer;
use crate::servers::game;
use crate::servers::game::UserSession;


#[get("/api/info")]
async fn game_info(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    println!("{:?}", extract_value(&req, "key"));

    let lobby_id = match extract_header_string(&req, "lobby-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };
    let lobby = srv.send(game::HasLobby { lobby_id: LobbyId::of(lobby_id.clone())}).await.expect("No Lobby found!");
    let error = json!(
        {
            "Error": "Lobby not found",
            "Lobby": lobby_id
        }
    );
    let user =match lobby {
        false => return Ok(HttpResponse::from(HttpResponse::InternalServerError().json(error))),
        true => "Found something"
    };
    Ok(HttpResponse::from(HttpResponse::Ok().json(user)))
}

#[derive(Debug, Clone,Serialize)]
struct UserSessionWithAdmin{
    user_session:UserSession,
    is_admin:bool
}

#[get("/api/session")]
async fn session_request(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>, auth: web::Data<Addr<AuthenticationServer>>) -> Result<HttpResponse, actix_web::Error> {
    let user_session = get_session(&req, &srv).await;
    let mut response = HttpResponse::from(HttpResponse::Ok().json(UserSessionWithAdmin{user_session:user_session.clone(), is_admin:is_admin(user_session.clone(), auth).await}));
    set_cookie(&mut response, &req,"user-session-id", &user_session.user_session_id.id.to_string());
    set_cookie(&mut response, &req,"session-token", &user_session.session_token.token);
    Ok(response)
}

pub async fn get_session(req: &HttpRequest, srv: &web::Data<Addr<game::GameServer>>) -> UserSession {
    let mut user_session_id = None;
    let mut session_token = None;
    if let Some(cookie) = req.cookie("user-session-id"){
        if let Ok(id) =  cookie.value().parse::<usize>(){
            user_session_id = Some(UserSessionId::of(id));
        }
    }
    if let Some(cookie) = req.cookie("session-token"){
        session_token = Some(SessionToken {
            token:cookie.value().to_string(),
            create: Local::now(),
        })
    };
    srv.send(GetUserSession{user_session_id, session_token}).await.expect("Something happens by getting the user")
}

pub fn get_token(req: &HttpRequest) -> Option<usize> {
    let cookie = match req.cookie("token") {
        None => return None,
        Some(cookie) => cookie,
    };
    match cookie.value().parse::<usize>() {
        Err(_) => None,
        Ok(id) => Some(id),
    }
}





pub fn set_cookie(res: &mut HttpResponse,req: &HttpRequest, cookie_name: &str, value: &String){
   //let expiration_time = SystemTime::now() + Duration::from_secs(60);
    let cookie = Cookie::build(cookie_name, value)
        .path("/")
        .secure(true)
        //TODO Do we need this?
        //.expires(Expiration::DateTime(OffsetDateTime::from(expiration_time)))
        .finish();
    if let Some(cookie) = req.cookie(cookie_name) {
        if(cookie.value().eq(value)) {
            return;
        }
    }
    let cookie = Cookie::build(cookie_name, value)
        .path("/")
        .secure(true)
        //TODO Do we need this?
        //.expires(Expiration::DateTime(OffsetDateTime::from(expiration_time)))
        .finish();
    println!("UPDATED!! {}", cookie_name.to_string());
    res.add_cookie(&cookie).expect("Can´t add cookies to the Response");
}

pub fn remove_cookie(res: &mut HttpResponse, req: &HttpRequest, cookie_name: &str){
    if let Some(cookie)= req.cookie(cookie_name) {
        res.add_removal_cookie(&cookie).expect("Can´t add cookies to the Response")
    }
}





#[get("/api/authorization")]
async fn has_authorization(_req: HttpRequest) -> HttpResponse {
    let board = JeopardyBoard::default(NORMAL);

    HttpResponse::from(HttpResponse::Ok().json(board))
}

#[post("/api/create")]
async fn create_game_lobby(_req: HttpRequest) -> HttpResponse {
    HttpResponse::from(HttpResponse::Ok())
}


#[post("/api/join")]
async fn join_game(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}


#[patch("/api/update-authorization")]
async fn update_authorization(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("true")
}


