use std::str::FromStr;
use crate::data::SessionRequest;
use crate::servers::game::{CreateLobby, GetUserAndUpdateSession, GetUserSession, SessionToken};
use actix::{Addr};

use actix_web::cookie::Cookie;
use actix_web::{get, HttpRequest, HttpResponse, post, web};
use chrono::Local;
use cult_common::wasm_lib::{ApiResponse, JeopardyMode};
use oauth2::http::header::COOKIE;
use oauth2::http::HeaderValue;
use serde::Serialize;
use serde_json::json;
use cult_common::backend::JeopardyBoard;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
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
    let lobby = srv.send(game::LobbyExists { lobby_id: LobbyId::of(lobby_id.clone())}).await.expect("No Lobby found!");
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
    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}

#[get("/api/session-data")]
async fn session_data_request(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>, auth: web::Data<Addr<AuthenticationServer>>) -> Result<HttpResponse, actix_web::Error> {
    let user_session = get_session(&req, &srv).await;
    let sessionrequest : SessionRequest = SessionRequest{
     user_session_id: user_session.user_session_id.clone(), 
     session_token: user_session.session_token.clone() 
    };

    let mut response = HttpResponse::from(HttpResponse::Ok().json(sessionrequest));
    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}


pub async fn get_updated_session(req: &HttpRequest, srv: &web::Data<Addr<game::GameServer>>) -> UserSession {
    let user_session_id = get_user_id_from_cookie(&req);
    let session_token = get_session_token_from_cookie(&req);
    srv.send(GetUserAndUpdateSession {user_session_id, session_token}).await.expect("Something happens by getting the user")
}

pub async fn get_updated_session_with_request(sessionRequest:SessionRequest, srv: &web::Data<Addr<game::GameServer>>) -> UserSession {
    let user_session_id = Some(sessionRequest.user_session_id);
    let session_token = Some(sessionRequest.session_token);
    srv.send(GetUserAndUpdateSession {user_session_id, session_token}).await.expect("Something happens by getting the user")
}

pub async fn get_session(req: &HttpRequest, srv: &web::Data<Addr<game::GameServer>>) -> UserSession {
    let user_session_id = get_user_id_from_value(&req).or(get_user_id_from_cookie(&req));
    let session_token = get_session_token_from_value(&req).or(get_session_token_from_cookie(&req));
    srv.send(GetUserSession {user_session_id, session_token}).await.expect("Something happens by getting the user")
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
    let cookie = format!("{}={}", cookie_name, value);
    res.headers_mut().append(COOKIE, HeaderValue::from_str(&cookie).unwrap());
    let _cookie = Cookie::build(cookie_name, value)
        .path("/")
        .permanent()
        .secure(true)
        .finish();
    /*if let Some(cookie) = req.cookie(cookie_name) {
        if cookie.value().eq(value) {
                return;
            }
    }
     println!("UPDATED!! {}", cookie_name.to_string() + "");
     */
    res.add_cookie(&_cookie).expect("Can´t add cookies to the Response");
}


pub fn set_session_token_cookie(response: &mut HttpResponse, req: &HttpRequest, user_session: &UserSession){
    set_cookie(response, &req, "user-session-id", &user_session.user_session_id.id.to_string());
    set_cookie(response, &req, "session-token", &user_session.session_token.token);
}






pub fn remove_cookie(res: &mut HttpResponse, req: &HttpRequest, cookie_name: &str){
    if let Some(cookie)= req.cookie(cookie_name) {
        res.add_removal_cookie(&cookie).expect("Can´t add cookies to the Response")
    }
}





#[get("/api/authorization")]
async fn has_authorization(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>, auth: web::Data<Addr<AuthenticationServer>>) -> HttpResponse {
    let user_session = get_session(&req, &srv).await;
    let mut response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::new(is_admin(user_session.clone(), auth).await)));
    set_session_token_cookie(&mut response, &req, &user_session);
    response
}

#[get("/api/discord_session")]
async fn discord_session(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>, _auth: web::Data<Addr<AuthenticationServer>>) -> HttpResponse {
    let user_session = get_session(&req, &srv).await;
    let discord_user = match user_session.clone().discord_auth{
        None => None,
        Some(data) => data.discord_user,
    };
    let mut response = HttpResponse::from(HttpResponse::Ok().json(discord_user));
    set_session_token_cookie(&mut response, &req, &user_session);
    response
}

#[post("/api/create")]
async fn create_game_lobby(req: HttpRequest,json: web::Json<Option<JeopardyBoard>>, srv: web::Data<Addr<game::GameServer>>, auth: web::Data<Addr<AuthenticationServer>>) -> HttpResponse {
    let user_session = get_session(&req, &srv).await;
    let mut response =   HttpResponse::from(HttpResponse::NotFound());
    if is_admin(user_session.clone(), auth).await{
        if let Some(discord_data) = user_session.clone().discord_auth {
            if let Some(discord_user) = discord_data.discord_user {
                let data = srv.send(CreateLobby { user_session_id: user_session.user_session_id.clone(), discord_id: discord_user.discord_id, jeopardy_board: json.into_inner() }).await.expect("Something happens by getting the user");
                response = HttpResponse::from(HttpResponse::Ok().json(data));
            }
        }
    }
    set_session_token_cookie(&mut response, &req, &user_session);
    response
}


#[get("/api/join")]
async fn join_game(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    println!("{:?}", extract_header_string(&req, "lobby-id"));
    let lobby_id = match extract_header_string(&req, "lobby-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };
    let user_session = get_session(&req, &srv).await;
    let can_join = srv.send(game::CanJoinLobby { user_session_id: user_session.user_session_id.clone(), lobby_id: LobbyId::of(lobby_id.clone())}).await.expect("No Lobby found!");

    let mut response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::new(can_join)));
    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}

#[get("/api/board")]
async fn board() -> HttpResponse {
    let response = HttpResponse::from(HttpResponse::Ok().json(JeopardyBoard::default(JeopardyMode::SHORT)));
    response
}


pub fn get_user_id_from_cookie(req: &HttpRequest) -> Option<UserSessionId> {
    if let Some(cookie) = req.cookie("user-session-id"){
        if let Ok(id) =  cookie.value().parse::<usize>(){
            return Some(UserSessionId::of(id));
        }
    }
    None
}

pub fn get_session_token_from_cookie(req: &HttpRequest) -> Option<SessionToken> {
    if let Some(cookie) = req.cookie("session-token"){
        return Some(SessionToken {
            token:cookie.value().to_string(),
            create: Local::now(),
        })
    };
    None
}

pub fn get_user_id_from_value(req: &HttpRequest) -> Option<UserSessionId> {
    if let Ok(cookie) = extract_value(&req,"user-session-id"){
        if let Ok(id) =  cookie.parse::<usize>(){
            return Some(UserSessionId::of(id));
        }
    }
    None
}

pub fn get_session_token_from_value(req: &HttpRequest) -> Option<SessionToken> {
    if let Ok(cookie) = extract_value(&req,"session-token"){
        return Some(SessionToken {
            token:cookie,
            create: Local::now(),
        })
    };
    None
}