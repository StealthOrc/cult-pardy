use std::time::{Duration, Instant};

use actix::prelude::*;

use crate::servers::game;
use crate::servers::game::{WebsocketDisconnect, SessionMessageType};
use actix_web::web;
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use chrono::{DateTime, Local, TimeDelta};
use cult_common::{
    compress, decompress, LobbyId, UserSessionId, WebsocketSessionEvent, WebsocketSessionId,
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsSession {
    pub player: UserData,
    pub hb: Instant,
    pub handler: Addr<game::GameServer>,
}

#[derive(Debug, Clone)]
pub struct UserData {
    pub websocket_session_id: Option<WebsocketSessionId>,
    pub user_session_id: UserSessionId,
    pub lobby_id: LobbyId,
    pub last_pong: DateTime<Local>,
    pub ping: i64
}

impl UserData {
    fn default(user_session_id: UserSessionId, lobby: LobbyId) -> Self {
        UserData {
            websocket_session_id: None,
            user_session_id,
            lobby_id: lobby,
            last_pong: Local::now(),
            ping: 0
        }
    }
}

impl WsSession {
    pub fn default(
        user_session_id: UserSessionId,
        lobby: LobbyId,
        srv: web::Data<Addr<game::GameServer>>,
    ) -> Self {
        WsSession {
            player: UserData::default(user_session_id, lobby),
            hb: Instant::now(),
            handler: srv.get_ref().clone(),
        }
    }

    fn hb(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping("".as_ref());
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);
        let addr = ctx.address();
        self.handler
            .send(game::WebsocketConnect {
                lobby_id: self.player.lobby_id.clone(),
                user_session_id: self.player.user_session_id.clone(),
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => match res {
                        None => {
                            println!("Something happens 3");
                            ctx.stop()
                        }
                        Some(id) => act.player.websocket_session_id = Some(id),
                    },

                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.handler.do_send(WebsocketDisconnect {
            user_data: self.player.clone(),
        });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<game::SessionMessageType> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: game::SessionMessageType, ctx: &mut Self::Context) {
        match msg {
            SessionMessageType::SelfDisconnect => ctx.stop(),
            SessionMessageType::Data(data) => {
                let binary = serde_json::to_vec(&data).expect("Can´t convert to vec");
                if let Ok(bytes) = compress(&binary) {
                    ctx.binary(bytes)
                }
            }
        }
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
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
            ws::Message::Pong(_) => {
                let current_ping = Local::now();
                self.player.last_pong = current_ping;
                self.player.ping = (current_ping.signed_duration_since(self.player.last_pong).num_milliseconds() - TimeDelta::seconds(5).num_milliseconds())/5;
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let text = text.trim();
                if text.starts_with('/') {
                    let v: Vec<&str> = text.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => handle_list_command(self, ctx),
                        _ => {}
                    }
                } else {
                    send_chat_message(self, text)
                }
            }
            ws::Message::Binary(data) => {
                if let Ok(bytes) = decompress(&data) {
                    match serde_json::from_slice::<WebsocketSessionEvent>(&bytes) {
                        Ok(event) => {
                            match event {
                                WebsocketSessionEvent::Click(vector2d) => {
                                    self.handler.do_send(game::LobbyClick {
                                        vector_2d: vector2d,
                                        user_data: self.player.clone(),
                                    });
                                },
                                WebsocketSessionEvent::Back => {
                                    self.handler.do_send(game::LobbyBackClick{
                                        user_data: self.player.clone(),
                                    });
                                }
                                WebsocketSessionEvent::AddUserSessionScore(grant_score_user_session_id) => {
                                    self.handler.do_send(game::AddLobbySessionScore{
                                        user_data: self.player.clone(),
                                        grant_score_user_session_id,
                                    });
                                }
                            }
                            println!("Receive an client event {:?}", event);
                        }
                        Err(err) => {
                            println!("Error deserializing JSON data:  {:?}", err);
                        }
                    }
                }
            }
            ws::Message::Close(reason) => {
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

fn handle_list_command(handler: &mut WsSession, ctx: &mut WebsocketContext<WsSession>) {
    println!("Listing rooms...");
    // Send ListRooms message to chat server and handle response asynchronously
    let fut = handler
        .handler
        .send(game::ListRooms)
        .into_actor(handler)
        .then(|res, _, ctx| {
            match res {
                Ok(rooms) => {
                    for room in rooms {
                        ctx.text(room);
                    }
                }
                Err(_) => println!("Failed to list rooms"),
            }
            fut::ready(())
        });
    ctx.wait(fut);
}

fn send_chat_message(handler: &mut WsSession, msg: &str) {
    // Send message to chat server
    handler.handler.do_send(game::ClientMessage {
        player_data: handler.player.clone(),
        msg: msg.to_owned(),
    });
}

