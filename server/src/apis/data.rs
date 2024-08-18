
use std::sync::Arc;
use crate::data::SessionRequest;
use crate::services::db::MongoServer;
use crate::services::game::SessionToken;

use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponse, web};
use chrono::Local;
use cult_common::wasm_lib::hashs::validate::ValidateHash;
use oauth2::http::header::COOKIE;
use oauth2::http::HeaderValue;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use crate::services::game::UserSession;

use super::error::{ApiRequestError, ErrorType, ToApiError, ToResponse};


pub fn get_internal_server_error_json(body: Value) -> HttpResponse {
    HttpResponse::InternalServerError().json(body)
}


pub fn extract_value(req: &HttpRequest, key: &str) -> Result<String, HttpResponse> {
    let query_string = req.query_string();
    if query_string.is_empty() {
        return Err(
            ApiRequestError::Error(key.to_string(), ErrorType::Missing).to_api_error().to_response()
        );
    }
    for pair in query_string.split('&') {
        let mut parts = pair.split('=');
        if let Some(k) = parts.next() {
            if k == key {
                if let Some(v) = parts.next() {
                    return Ok(v.to_string());
                }
            }
        }
    }
    return Err(
        ApiRequestError::Error(key.to_string(), ErrorType::Missing).to_api_error().to_response()
    );
}

pub fn extract_header_string(req: &HttpRequest, header_name: &str) -> Result<String, HttpResponse> {
    match req.headers().get(header_name) {
        None => Err(ApiRequestError::Error(header_name.to_string(), ErrorType::Missing).to_api_error().to_response()),
        Some(value) => {
            if value.is_empty() {
                Err(ApiRequestError::Error(header_name.to_string(), ErrorType::Empty).to_api_error().to_response())
            } else {
                match value.to_str() {
                    Ok(text) => Ok(text.to_string()),
                    Err(_) => Err(HttpResponse::InternalServerError()
                        .json(ApiRequestError::Error(header_name.to_string(), ErrorType::StringConversion).to_api_error())),
                }
            }
        }
    }
}

pub fn get_lobby_id_from_header(req: &HttpRequest) -> Option<LobbyId> {
    match extract_header_string(req, "lobby_id") {
        Ok(lobby_id) => Some(LobbyId::of(lobby_id)),
        Err(_) => None,
    }
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
            expire: Local::now(),
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
            expire: Local::now(),
        })
    };
    None
}


pub fn get_lobby_id_from_value(req: &HttpRequest) -> Option<LobbyId> {
    if let Ok(cookie) = extract_value(&req,"lobby-id"){
        return Some(LobbyId::of(cookie));
    }
    None
}



pub fn get_file_name_from_value(req: &HttpRequest) -> Option<String> {
    if let Ok(cookie) = extract_value(&req,"file-name"){
        return Some(cookie);
    };
    None
}

pub fn get_file_index_from_value(req: &HttpRequest) -> Option<usize> {
    if let Ok(cookie) = extract_value(&req,"file-index"){
        if let Ok(id) =  cookie.parse::<usize>(){
            return Some(id);
        }
    }
    None
}

pub fn get_validate_hash_from_value(req: &HttpRequest) -> Option<ValidateHash> {
    if let Ok(cookie) = extract_value(&req,"validate-hash"){
        return Some(ValidateHash::new(cookie));
    }
    None
}

pub async fn get_session_with_token_update_or_create_new(req: &HttpRequest, db: &web::Data<Arc<MongoServer>>) -> UserSession {
    let user_session_id_cookie = get_user_id_from_cookie(&req);
    let user_session_id_value = get_user_id_from_value(&req);
    
    


    //println!("User-session-id-value: {:?} User-session-id-cookie: {:?}", user_session_id_value.clone().unwrap_or(UserSessionId::server()).id, user_session_id_cookie.clone().unwrap_or(UserSessionId::server()).id);

    let user_session_id = match user_session_id_value.or(user_session_id_cookie) {
        Some(data) => data,
        None =>  {
            println!("User-session-id cookie not found");
            return db.new_user_session().await
        }
    };

    let session_token_cookie = get_session_token_from_cookie(&req);
    let session_token_value = get_session_token_from_value(&req);
    // println!("Session-token-value: {:?} Session-token-cookie: {:?}", session_token_value.clone().unwrap_or(SessionToken::server()).token, session_token_cookie.clone().unwrap_or(SessionToken::server()).token);

    let session_token = match session_token_value.or(session_token_cookie) {
        Some(data) => data,
        None =>  {
            println!("Session-cookie not found");
            return db.new_user_session().await
        }
    };
    db.get_user_session_with_token_check(&user_session_id, &session_token).await
}


pub async fn get_session_or_create_new_session_request(session_request:&SessionRequest, db: &web::Data<Arc<MongoServer>>) -> UserSession {
    db.get_user_session_with_token_check(&session_request.user_session_id, &session_request.session_token).await

}



pub async fn get_session(req: &HttpRequest, db: &web::Data<Arc<MongoServer>>) -> Option<UserSession> {
    let user_session_id = match get_user_id_from_value(&req).or(get_user_id_from_cookie(&req)) {
        Some(data) => data,
        None => {
            println!("User-session-id cookie not found");
            return None
        }
    };
    let session_token = match get_session_token_from_value(&req).or(get_session_token_from_cookie(&req)) {
        Some(data) => data,
        None => {
            println!("Session-cookie not found");
            return None
        }
    };
    db.get_user_session_with_id(&user_session_id, &session_token).await

}



pub async fn find_session(req: &HttpRequest, db: &web::Data<Arc<MongoServer>>) -> Option<UserSession> {
    let user_session_id = match get_user_id_from_value(&req).or(get_user_id_from_cookie(&req)) {
        Some(data) => data,
        None => {
            println!("User-session-id cookie not found");
            return None
        }
    };
    let session_token = match get_session_token_from_value(&req).or(get_session_token_from_cookie(&req)) {
        Some(data) => data,
        None => {
            println!("Session-cookie not found");
            return None
        }
    };
    
    let user = match db.find_user_session(&user_session_id).await {
        Some(data) => data,
        None => {
            println!("User-session not found");
            return None
        }
    };
    if user.session_token.token.eq(&session_token.token) {
        return Some(user);
    }
    None

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





pub fn set_cookie(res: &mut HttpResponse, cookie_name: &str, value: &String){
    let cookie = format!("{}={}", cookie_name, value);
    res.headers_mut().append(COOKIE, HeaderValue::from_str(&cookie).unwrap());
    let _cookie = Cookie::build(cookie_name, value)
        .path("/")
        .permanent()
        .secure(true)
        .finish();
    res.add_cookie(&_cookie).expect("Can´t add cookies to the Response");
}


pub fn set_session_token_cookie(response: &mut HttpResponse, user_session: &UserSession){
    set_cookie(response, "user-session-id", &user_session.user_session_id.id.to_string());
    set_cookie(response, "session-token", &user_session.session_token.token);
}






pub fn remove_cookie(res: &mut HttpResponse, req: &HttpRequest, cookie_name: &str){
    if let Some(cookie)= req.cookie(cookie_name) {
        res.add_removal_cookie(&cookie).expect("Can´t add cookies to the Response")
    }
}

