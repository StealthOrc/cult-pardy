use std::sync::Arc;
use std::time::{Duration, Instant};
use std::vec;

use actix::prelude::*;
use bytes::{Bytes, BytesMut};
use cult_common::{compress, decompress};
use serde::{Deserialize, Serialize};

use crate::servers::game::{self, GameServer};
use crate::servers::lobby::{AddLobbySessionScore, ClientMessage, Lobby, LobbyBackClick, LobbyClick, ReciveVideoEvent, UpdateWebsocketPing, WebsocketConnect, WebsocketDisconnect};
use actix_web::web;
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use chrono::{DateTime, Local, TimeDelta};
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use cult_common::wasm_lib::ids::websocketsession::WebsocketSessionId;
use cult_common::wasm_lib::websocket_events::{SessionEvent, WebsocketServerEvents, WebsocketSessionEvent};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);



#[derive(Debug)]
pub struct FileSession {
    pub hb: Instant,
    pub file_chunks: BytesMut,
}





impl FileSession {

        pub fn default() -> Self {
            FileSession {
                hb: Instant::now(),
                file_chunks: BytesMut::new(),
            }
        }


        fn hb(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
            ctx.run_interval(HEARTBEAT_INTERVAL, |act: &mut FileSession, ctx| {
                let time_since = Instant::now().duration_since(act.hb);
                if time_since > CLIENT_TIMEOUT {
                    println!("Websocket Client heartbeat failed, disconnecting!");
                    ctx.stop();
                    return;
                }
                ctx.ping(b"");
            });

        }

    }


impl Actor for FileSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }

  
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}







/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for FileSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };
        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) =>    self.hb = Instant::now(),
            ws::Message::Text(_) => todo!(),
            ws::Message::Binary(data) => {
                self.file_chunks.extend_from_slice(&data);
                ctx.text("ack");
                
            }
            ws::Message::Close(reason) => {
                println!("Received {} bytes", self.file_chunks.len());
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}


