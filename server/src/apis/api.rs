
use crate::servers::game::{LobbyId};
use actix::Addr;

use actix_web::cookie::{Cookie};
use actix_web::{get, HttpRequest, HttpResponse, patch, post, web};
use serde::Serialize;
use serde_json::json;
use cult_common::{JeopardyBoard, UserSessionId};
use cult_common::JeopardyMode::NORMAL;
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
    set_cookie(
        &mut response,
        "user-session-id",
        &user_session.user_session_id.id.to_string(),
    );
    Ok(response)
}

pub async fn get_session(req: &HttpRequest, srv: &web::Data<Addr<game::GameServer>>) -> UserSession {
    let user_req = match req.cookie("user-session-id") {
        None => None,
        Some(cookie) => match cookie.value().parse::<usize>() {
            Err(_) => None,
            Ok(id) => Some(UserSessionId{ id }),
        }
    };

    println!("!!{:?}", user_req);
    srv.send(game::GetUserSession {user_session_request:user_req}).await.expect("Somethings wrong with sessions")
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





pub fn set_cookie(res: &mut HttpResponse, cookie_name: &str, cookie: &str){
   //let expiration_time = SystemTime::now() + Duration::from_secs(60);
    let cookie = Cookie::build(cookie_name, cookie)
        .path("/")
        .secure(true)
        //TODO Do we need this?
        //.expires(Expiration::DateTime(OffsetDateTime::from(expiration_time)))
        .finish();
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


