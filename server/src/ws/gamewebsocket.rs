use std::sync::Arc;

use actix::{Addr};
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use chrono::Local;
use serde_json::json;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use crate::apis::api::{find_session, get_lobby_id_from_value};
use crate::apis::data::{self, extract_value, get_internal_server_error_json};
use crate::servers::db::MongoServer;
use crate::servers::game::{self, GameServer};
use crate::servers::game::SessionToken;
use crate::ws::session::WsSession;

pub async fn start_ws(req: HttpRequest, stream: web::Payload, db: web::Data<Arc<MongoServer>>, game: web::Data<Addr<GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    //TODO MAKE MATCHES GREAT AGAIN!user_session_id



    let lobby_id = match get_lobby_id_from_value(&req){
        Some(data) => data,
        None => return Ok(get_internal_server_error_json(json!({"Error": "No Lobby ID Found"}))),
    };



    let user = match find_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(get_internal_server_error_json(json!({"Error": "No User Session Found"}))),
    };
    


    let loddy_addr = match game.send(game::LobbyAddrRequest{lobby_id:lobby_id.clone()}).await.expect("No Lobby found!") {
        Some(data) => data,
        None => return Ok(get_internal_server_error_json(json!({"Error": "No Lobby Found"}))),
    };

    
    ws::start(WsSession::default(&user.user_session_id, &lobby_id, &game, &loddy_addr), &req, stream)

}

