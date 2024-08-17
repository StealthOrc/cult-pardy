use std::sync::Arc;

use actix::{Addr};
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use serde_json::json;
use crate::apis::data::{find_session, get_internal_server_error_json, get_lobby_id_from_value};
use crate::services::db::MongoServer;
use crate::services::game::{self, GameServer};
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

