

use actix::Addr;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use serde_json::json;
use cult_common::UserSessionId;
use crate::apis::data::{extract_value, get_internal_server_error_json};
use crate::servers::game;
use crate::servers::game::{LobbyId};
use crate::ws::session::WsSession;

pub async fn start_ws(req: HttpRequest, stream: web::Payload, srv: web::Data<Addr<game::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    //TODO MAKE MATCHES GREAT AGAIN!user_session_id
    println!("TEST?");

    let session_token = match extract_value(&req, "user-session-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };

    let lobby_id = match extract_value(&req, "lobby-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };


    let lobbies = srv.send(game::Lobbies).await.expect("No Lobbies found");


    let error = json!(
        {
            "Error": "Lobby not found",
            "Lobby": lobby_id
        }
    );
    if !lobbies.contains(&lobby_id) { return Ok(get_internal_server_error_json(error)); }

    println!("{} - {}", lobby_id, session_token);
    ws::start(
        WsSession::default(UserSessionId::from_string(session_token), LobbyId::of(lobby_id), srv),
        &req,
        stream,)

}

