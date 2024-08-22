use std::time::{Duration, Instant};

use actix::prelude::*;
use cult_common::{compress, decompress};
use serde::{Deserialize, Serialize};

use crate::services::game::{self};
use crate::services::lobby::{AddLobbySessionScore, Lobby, LobbyBackClick, LobbyClick, ReciveVideoEvent, SyncBackwardRequest, SyncForwardRequest, UpdateWebsocketPing, WebsocketConnect, WebsocketDisconnect};
use actix_web::web;
use actix_web_actors::ws;
use chrono::{DateTime, Local};
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use cult_common::wasm_lib::ids::websocketsession::WebsocketSessionId;
use cult_common::wasm_lib::websocket_events::{WebsocketEvent, WebsocketServerEvents, WebsocketSessionEvent};

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
    pub game_server_addr: Addr<game::GameServer>,
    pub lobby_addr:Addr<Lobby>
}

#[derive(Debug, Clone)]
pub struct UserData {
    pub websocket_session_id: Option<WebsocketSessionId>,
    pub user_session_id: UserSessionId,
    pub lobby_id: LobbyId,
    pub ping: i64,
}

impl UserData {
    fn default(user_session_id: UserSessionId, lobby: LobbyId) -> Self {
        UserData {
            websocket_session_id: None,
            user_session_id,
            lobby_id: lobby,
            ping: 0,
        }
    }
}

impl WsSession {
        pub fn default(user_session_id: &UserSessionId,lobby_id: &LobbyId, srv: &web::Data<Addr<game::GameServer>>, lobby: &Addr<Lobby>) -> Self {
            WsSession {
                player: UserData::default(user_session_id.clone(), lobby_id.clone()),
                hb: Instant::now(),
                game_server_addr: srv.get_ref().clone(),
                lobby_addr: lobby.clone(),
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
            });

        }

        fn is_available(&self) -> bool {
          self.player.websocket_session_id.is_some()
        }

        fn set_ping(&mut self, _: &mut ws::WebsocketContext<Self>) {
            if let Some(websocket_id) = &self.player.websocket_session_id {
                    self.lobby_addr.do_send(UpdateWebsocketPing {
                        websocket_session_id: websocket_id.clone(),
                        ping: self.player.ping,
                    });
                
            }
        }


    fn ping(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        let time = Local::now();
        let test = serde_json::to_vec(&time).expect("Can´t convert to vec");
        ctx.ping(&test.as_slice());
    
    }

    fn get_websocket_session(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        self.lobby_addr.send(WebsocketConnect {
            user_session_id: self.player.user_session_id.clone(),
            addr: ctx.address().recipient(),
            ping: self.player.ping,
        })
        .into_actor(self)
        .then(|res, act, ctx| {
            match res {
                Ok(res) => match res {
                    None => {
                        println!("Something happens 2");
                        ctx.stop()
                    },
                    Some(websocket_session_id) => {
                      if let Ok(bytes) = compress(&serde_json::to_vec(&WebsocketServerEvents::Websocket(WebsocketEvent::WebsocketID(websocket_session_id.clone()))).expect("Can´t convert to vec")) {
                            ctx.binary(bytes)
                     }
                      act.player.websocket_session_id = Some(websocket_session_id);
                    }
                },
                _ => ctx.stop(),
            }
            fut::ready(())
        })
        .wait(ctx);
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.get_websocket_session(ctx);
    }

  
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.lobby_addr.do_send(WebsocketDisconnect {
                user_data: self.player.clone(),
        });
        Running::Stop
    }
}



#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub enum SendSessionMessageType {
    Data(WebsocketServerEvents),
    SelfDisconnect,
}

impl Handler<SendSessionMessageType> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: SendSessionMessageType, ctx: &mut Self::Context) -> Self::Result {
        match msg  {
            SendSessionMessageType::SelfDisconnect => ctx.stop(),
            SendSessionMessageType::Data(data) => {
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

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(bytes) => {
                if !bytes.is_empty() {
                    if let Ok(pong) = serde_json::from_slice::<DateTime<Local>>(&bytes) {
                        let time = Local::now();

                        let ping = time.signed_duration_since(pong).num_milliseconds();
                        self.player.ping = ping;
                        self.set_ping(ctx);
                    }
                }
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let _ = text.trim();
                //send_chat_message(self, text)
            }
            ws::Message::Binary(data) => {
                    //TODO: make deflate alogithm de-/activatable again for development
                    if let Ok(bytes) = decompress(&data) {
                        match serde_json::from_slice::<WebsocketSessionEvent>(&bytes) {
                            Ok(event) => {
                                match event.clone() {
                                    WebsocketSessionEvent::Click(vector2d) => {
                                        self.lobby_addr.do_send(LobbyClick {
                                            vector_2d: vector2d,
                                            user_data: self.player.clone(),
                                        });
                                    },
                                    WebsocketSessionEvent::Back => {
                                        self.lobby_addr.do_send(LobbyBackClick{
                                            user_data: self.player.clone(),
                                        });
                                    }
                                    WebsocketSessionEvent::AddUserSessionScore(grant_score_user_session_id,  vector2d) => {
                                        self.lobby_addr.do_send(AddLobbySessionScore{
                                            user_data: self.player.clone(),
                                            grant_score_user_session_id,
                                            vector2d
                                        });
                                    }
                                    WebsocketSessionEvent::VideoEvent(event) => {
                                        let id = match self.player.websocket_session_id.clone() {
                                            Some(data) => data,
                                            None => return,
                                        };
                               
                                        self.lobby_addr.do_send(ReciveVideoEvent{
                                        user_session_id: self.player.user_session_id.clone(),
                                        websocket_session_id: id,
                                        event
                                    })
                                    }
                                    WebsocketSessionEvent::SyncBackwardRequest => {
                                        let id = match self.player.websocket_session_id.clone() {
                                            Some(data) => data,
                                            None => return,
                                        };
                                        self.lobby_addr.do_send(SyncBackwardRequest{
                                            websocket_session_id: id,
                                        });
                                    }
                                    WebsocketSessionEvent::SyncForwardRequest(time) => {
                                        let id = match self.player.websocket_session_id.clone() {
                                            Some(data) => data,
                                            None => return,
                                        };
                                        self.lobby_addr.do_send(SyncForwardRequest{
                                            websocket_session_id: id,
                                            current_time: time,
                                        });
                                    }
                                }
                                //println!("Receive an client event {:?}", event);
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


