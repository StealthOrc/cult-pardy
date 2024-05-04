use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::web;
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use cult_common::{LobbyId, UserSessionId, WebsocketSessionId};
use crate::servers::game;
use crate::servers::game::{SessionDisconnect, SessionMessageType};


/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


#[derive(Debug)]
pub struct WsSession{
    pub player: UserData,
    pub hb: Instant,
    pub handler: Addr<game::GameServer>,
}

#[derive(Debug, Clone)]
pub struct UserData {
    pub websocket_session_id: Option<WebsocketSessionId>,
    pub user_session_id: UserSessionId,
    pub lobby_id: LobbyId,
}

impl UserData {
    fn default(user_session_id: UserSessionId, lobby: LobbyId) -> Self {
        UserData {
            websocket_session_id: None,
            user_session_id,
            lobby_id: lobby,
        }
    }
}





impl WsSession {

    pub fn default(user_session_id: UserSessionId, lobby: LobbyId, srv: web::Data<Addr<game::GameServer>>) -> Self{
        WsSession{
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
        self.handler.send(game::Connect { lobby_id: self.player.lobby_id.clone(), user_session_id: self.player.user_session_id.clone(), addr: addr.recipient(), })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        match res {

                            None => {
                                println!("Something happens 3");
                                ctx.stop()
                            },
                            Some(id) => act.player.websocket_session_id = Some(id)
                        }
                    },

                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        //notify chat server
        //println!("STOP!? id: {}", self.player.websocket_session_id.unwrap().id);

        self.handler.do_send(SessionDisconnect{
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
            SessionMessageType::SelfDisconnect => {
                ctx.stop()
            }
            SessionMessageType::Data(data) => {
                let json = serde_json::to_vec(&data).expect("Can´t convert to vec");
                ctx.binary(json)
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
            ws::Message::Binary(_) => println!("Unexpected binary"),
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
    let fut = handler.handler.send(game::ListRooms)
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