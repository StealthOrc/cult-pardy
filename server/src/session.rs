use std::any::Any;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::web;
use actix_web_actors::ws;
use actix_web_actors::ws::{CloseCode, CloseReason};
use serde::{Serialize, Deserialize};
use log::log;

use crate::server;
use crate::server::SessionDataType;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


#[derive(Debug)]
pub struct WsSession {
    pub player: PlayerData,
    pub hb: Instant,
    pub handler: Addr<server::GameServer>,
}

#[derive(Debug, Clone)]
pub struct PlayerData {
    pub id: Option<usize>,
    pub lobby: String,
    pub name: String,
}

impl PlayerData {
    fn default(lobby:String, name:String) -> Self {
        PlayerData {
            id: None,
            lobby,
            name,
        }
    }
}





impl WsSession {

    pub fn default(lobby:String, name: String, srv: web::Data<Addr<server::GameServer>>) -> Self{
        WsSession{
            player: PlayerData::default(lobby, name),
            hb: Instant::now(),
            handler: srv.get_ref().clone(),
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");
                act.handler.do_send(server::Disconnect { id: act.player.id.unwrap() });
                ctx.stop();
                return;
            }
            ctx.ping(b"");
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
            .send(server::Connect { addr: addr.recipient(), })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.player.id = Some(res);
                        println!("RES!? id: {}", act.player.id.unwrap());
                    },

                    // something is wrong with chat server



                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        println!("STOP!? id: {}", self.player.id.unwrap());
        self.handler.do_send(server::Disconnect { id: self.player.id.unwrap() });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::SessionDataType> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: server::SessionDataType, ctx: &mut Self::Context) {
        match msg {
            SessionDataType::MText(text) => {
                ctx.text(text)}
            SessionDataType::MData(data) => {
                let json = serde_json::to_vec(&data).expect("Can´t convert to vec");
                ctx.binary(json)}
            SessionDataType::Disconnect => {
                println!("STOPED!");
                ctx.stop()
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
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            println!("List rooms");
                            self.handler
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 2 {
                                self.player.lobby = v[1].to_owned();
                                self.handler.do_send(server::Join { playerdata: self.player.clone() });
                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.player.name = v[1].to_owned();
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {
                    let msg = if let Some(ref name) = self.player.id {
                        format!("{name}: {m}");
                    } else {
                        let _ = m.to_owned();
                    };
                    // send message to chat server
                    self.handler.do_send(server::ClientMessage {
                        player_data: self.player.clone(),
                        msg: "".to_string(),
                    })
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


