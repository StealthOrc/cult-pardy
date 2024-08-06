use std::any;
use std::hash::Hash;
use std::str::FromStr;
use crate::data::SessionRequest;
use crate::main;
use crate::servers::db::{DBDatabase, MongoServer};
use crate::servers::game::{CreateLobby, GetUserAndUpdateSession, GetUserSession, SessionToken};
use actix::{Addr};

use actix_web::cookie::Cookie;
use actix_web::{get, HttpRequest, HttpResponse, post, web};
use attohttpc::body::File;
use chrono::Local;
use cult_common::dto::{DTOFileChunk, DTOFileData};
use cult_common::wasm_lib::{ApiResponse, JeopardyMode, FileData};
use mongodb::change_stream::session;
use oauth2::http::header::COOKIE;
use oauth2::http::{response, HeaderValue};
use serde::Serialize;
use serde_json::json;
use cult_common::backend::JeopardyBoard;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use crate::apis::data::{extract_header_string, extract_value, get_internal_server_error_json};
use crate::authentication::discord::is_admin;
use crate::servers::authentication::AuthenticationServer;
use crate::servers::{db, game};
use crate::servers::game::UserSession;

use super::data;


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

#[post("/api/upload/filedata")]
async fn upload_file_data(req: HttpRequest,  db: web::Data<MongoServer> ,srv: web::Data<Addr<game::GameServer>>, auth: web::Data<Addr<AuthenticationServer>>,  json: web::Json<DTOFileData>) -> Result<HttpResponse, actix_web::Error> {
    println!("UPLOAD FILE DATA");
    let user_session = get_session(&req, &srv).await;
    let mut response;
    if is_admin(user_session.clone(), auth).await == false {
        response = get_internal_server_error_json({
            json!({
                "Error": "Unauthorized",
                "User": user_session.user_session_id.id.to_string()
            })
        });
        set_session_token_cookie(&mut response, &req, &user_session);
        return Ok(response)
    } 


    let dto_filedata = json.into_inner();
    println!("DATA HASH {:?} ", dto_filedata.validate_hash);




    if db.is_file_data(&dto_filedata.file_name) {
        return Ok(session_error(&req, &user_session, "File already exists"))
    }



    let mut response;
    let file = dto_filedata.clone().to_file_data(&user_session.user_session_id);
    if db.add_file_data(FileData::from(file.clone())).await{
        response = HttpResponse::from(HttpResponse::Ok().json(file));
    }else{
        return Ok(session_error(&req, &user_session, "Can´t add file data"));
    }
    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}


pub static mut test: Option<FileData> = None;



#[post("/api/upload/filechunk")]
async fn upload_file_chunk(req: HttpRequest,  db: web::Data<MongoServer> ,srv: web::Data<Addr<game::GameServer>>, auth: web::Data<Addr<AuthenticationServer>>, json: web::Json<DTOFileChunk>) -> Result<HttpResponse, actix_web::Error> {
    println!("UPLOAD FILE CHUNK");
    let user_session = get_session(&req, &srv).await;
    let mut response;
    if is_admin(user_session.clone(), auth).await == false {
        return Ok(session_error(&req, &user_session, "Unauthorized"));
    } 

    let dto_filechunk = json.into_inner();

    let file_chunk = dto_filechunk.clone().to_file_chunk();
    let name = file_chunk.file_name.clone();
    let hash = file_chunk.filechunk_hash.clone();

    println!("HASH: {:?} - Validate: {:?}", hash, file_chunk.validate_hash.get_hash());



    if file_chunk.validate_hash.validate_file_chunk(&hash) == false {
        return Ok(session_error(&req, &user_session, "Invalid File Chunk"));
    }

    match db.get_file_chunks_by_index(&name, &file_chunk.index){
        Some(found_chunk) => {
            println!("File chunk already exists {} - {}", file_chunk.file_name, found_chunk.file_name);
            return Ok(session_error(&req, &user_session, "File chunk already exists"));
        },
        None => (),
    }

    if db.is_file_chunk_valide(&name, &hash) == false {
        return Ok(session_error(&req, &user_session, "File chunk is not valid"));
    }

    if db.add_file_chunk(&file_chunk).await{
        response = HttpResponse::from(HttpResponse::Ok().json(file_chunk));
        if db.is_last_file_chunk(&name){
            println!("LAST CHUNK");
        }
    }
    else{
       response = session_error(&req, &user_session, "Can´t add file chunk");
    }

    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}





pub fn session_error(req: &HttpRequest, user_session:&UserSession, str:&str) -> HttpResponse {
    let mut response = HttpResponse::InternalServerError().json(json!({
        "Error": str
    }));
    set_session_token_cookie(&mut response, &req, &user_session);
    response
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