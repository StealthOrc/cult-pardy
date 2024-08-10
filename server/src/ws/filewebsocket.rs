use std::sync::Arc;

use actix::{Addr};
use actix_files::Files;
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

use super::files::FileSession;

pub async fn start_ws(req: HttpRequest, stream: web::Payload, db: web::Data<Arc<MongoServer>>) -> Result<HttpResponse, actix_web::Error> {



    println!("Starting File WS");




    
    ws::start(FileSession::default(), &req, stream)

}

