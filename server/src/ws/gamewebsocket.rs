use actix::{Addr};
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use chrono::Local;
use serde_json::json;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use crate::apis::data::{extract_value, get_internal_server_error_json};
use crate::servers::game;
use crate::servers::game::SessionToken;
use crate::ws::session::WsSession;

pub async fn start_ws(req: HttpRequest, stream: web::Payload, srv: web::Data<Addr<game::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    //TODO MAKE MATCHES GREAT AGAIN!user_session_id

    let session_id = match extract_value(&req, "user-session-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };


    let lobby_id = match extract_value(&req, "lobby-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };

    let token = match extract_value(&req, "session-token") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };

    let request_session_id= UserSessionId::from_string(session_id.clone());

    let user_session = match srv.send(game::GetUserSession {
        user_session_id: Some(request_session_id),
        session_token: Some(SessionToken {
            token,
            create: Local::now(),
        }),
    }).await {
        Ok(data) => data,
        Err(_error) => return Ok(HttpResponse::NotFound().finish()),
    };


    let lobbies = srv.send(game::Lobbies).await.expect("No Lobbies found");


    let error = json!(
        {
            "Error": "Lobby not found",
            "Lobby": lobby_id
        }
    );
    if !lobbies.contains(&lobby_id) { return Ok(get_internal_server_error_json(error)); }

    ws::start(WsSession::default(user_session.user_session_id.clone(), LobbyId::of(lobby_id), srv), &req, stream)

}

