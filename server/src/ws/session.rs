use std::time::{Duration, Instant};
use std::vec;

use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::servers::game::{self, GameServer, SendSessionMessageType, SessionMessageResult};
use crate::servers::game::{WebsocketDisconnect, SessionMessageType};
use actix_web::web;
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use chrono::{DateTime, Local, TimeDelta};
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use cult_common::wasm_lib::ids::websocketsession::WebsocketSessionId;
use cult_common::wasm_lib::websocketevents::{SessionEvent, WebsocketServerEvents, WebsocketSessionEvent};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct GetPing {

}

impl Message for GetPing {
    type Result = i32;
    
}


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
    pub ping: i64,
    pub last_ping: DateTime<Local>,
}

impl UserData {
    fn default(user_session_id: UserSessionId, lobby: LobbyId) -> Self {
        UserData {
            websocket_session_id: None,
            user_session_id,
            lobby_id: lobby,
            ping: 0,
            last_ping: Local::now(),
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
        ctx.run_interval(HEARTBEAT_INTERVAL, |act: &mut WsSession, ctx| {
            let time_since = Instant::now().duration_since(act.hb);
            if time_since > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            act.ping(ctx);
            act.get_pings(ctx)
        });

    }

    fn set_ping(&mut self, _: &mut ws::WebsocketContext<Self>) {
        if let Some(websocket_id) = &self.player.websocket_session_id {
            self.handler
                .do_send(game::UpdateWebsocketsPing {
                    lobby_id: self.player.lobby_id.clone(),
                    websocket_session_id: websocket_id.clone(),
                    ping: self.player.ping,
                });

        }

    }



    fn get_pings(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        self.handler
        .send(game::GetSessionsPings {
            lobby_id: self.player.lobby_id.clone(),
        })
        .into_actor(self)
        .then(|res, _, ctx| {
            match res {
                Ok(res) => {
                    if res.len() > 0 {
                        let binary: Vec<u8> = serde_json::to_vec(&WebsocketServerEvents::Session(SessionEvent::SessionsPing(res))).expect("Can´t convert to vec");
                        ctx.binary(binary);
                    }
                }
                _ => ctx.stop(),
            }
            fut::ready(())
        })
        .wait(ctx);
    }


    fn ping(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        let time = Local::now();
        self.player.last_ping = time;
        println!("Send ping {:?}", time.timestamp_millis());
        let test = serde_json::to_vec(&time).expect("Can´t convert to vec");
        ctx.ping(&test.as_slice());
    
    }


}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let addr = ctx.address();
        self.handler
            .send(game::WebsocketConnect {
                lobby_id: self.player.lobby_id.clone(),
                user_session_id: self.player.user_session_id.clone(),
                addr: addr.recipient(),
                ping: self.player.ping,
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => match res {
                        None => {
                            println!("Something happens 3");
                            ctx.stop()
                        }
                        Some(id) => {
                            act.player.websocket_session_id = Some(id);
                            act.ping(ctx);
                        }
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
impl Handler<SessionMessageType> for WsSession {
    type Result = MessageResult<SessionMessageType>;

    fn handle(&mut self, msg: game::SessionMessageType, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SessionMessageType::Send(send) => 
            {
                match send  {
                    SendSessionMessageType::SelfDisconnect => ctx.stop(),
                    SendSessionMessageType::Data(data) => {
                        let binary = serde_json::to_vec(&data).expect("Can´t convert to vec");
                        ctx.binary(binary);
                        //TODO: make deflate alogithm de-/activatable again for development
                        //if let Ok(bytes) = compress(&binary) {
                        //    ctx.binary(bytes)
                        //}
                    }
                }
                MessageResult(SessionMessageResult::Void)
            }
            SessionMessageType::Get(get) => {
                match get {
                    game::GetSessionMessageType::GetPing => { 
                        MessageResult(SessionMessageResult::U64(self.player.ping as u64))
                    }
                }
            },
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
            ws::Message::Pong(bytes) => {
                if !bytes.is_empty() {
                    if let Ok(pong) = serde_json::from_slice::<DateTime<Local>>(&bytes) {
                        let time = Local::now();

                        println!("rec + cur + ping {:?} {:?} {:?}",pong.timestamp_millis(), time.timestamp_millis(), time.signed_duration_since(pong).num_milliseconds());
                        let ping = time.signed_duration_since(pong).num_milliseconds();
                        self.player.ping = ping;
                        self.set_ping(ctx);
                    }
                }
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let text = text.trim();
                send_chat_message(self, text)
            }
            ws::Message::Binary(data) => {
                //TODO: make deflate alogithm de-/activatable again for development
                //if let Ok(bytes) = decompress(&data) {
                    match serde_json::from_slice::<WebsocketSessionEvent>(&data) {
                        Ok(event) => {
                            match event.clone() {
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
                                WebsocketSessionEvent::AddUserSessionScore(grant_score_user_session_id,  vector2d) => {
                                    self.handler.do_send(game::AddLobbySessionScore{
                                        user_data: self.player.clone(),
                                        grant_score_user_session_id,
                                        vector2d
                                    });
                                }
                            }
                            println!("Receive an client event {:?}", event);
                        }
                        Err(err) => {
                            println!("Error deserializing JSON data:  {:?}", err);
                        }
                    }
                //}
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



fn send_chat_message(handler: &mut WsSession, msg: &str) {
    // Send message to chat server
    handler.handler.do_send(game::ClientMessage {
        player_data: handler.player.clone(),
        msg: msg.to_owned(),
    });
}

