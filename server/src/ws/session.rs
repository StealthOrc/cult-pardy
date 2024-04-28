
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::web;
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use crate::servers::game;
use crate::servers::game::SessionDataType;


/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


#[derive(Debug)]
pub struct WsSession {
    pub player: PlayerData,
    pub hb: Instant,
    pub handler: Addr<game::GameServer>,
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

    pub fn default(lobby:String, name: String, srv: web::Data<Addr<game::GameServer>>) -> Self{
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
                act.handler.do_send(game::Disconnect { id: act.player.id.unwrap() });
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
            .send(game::Connect { addr: addr.recipient(), })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.player.id = Some(res);
                        println!("RES!? id: {}", act.player.id.unwrap());
                    },

                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        println!("STOP!? id: {}", self.player.id.unwrap());
        self.handler.do_send(game::Disconnect { id: self.player.id.unwrap() });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<game::SessionDataType> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: game::SessionDataType, ctx: &mut Self::Context) {
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
                let text = text.trim();
                if text.starts_with('/') {
                    let v: Vec<&str> = text.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => handle_list_command(self, ctx),
                        "/join" => handle_join_command(self, v, ctx),
                        "/name" => handle_name_command(self, v, ctx),
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

fn handle_join_command(handler: &mut WsSession, room_name: Vec<&str>, ctx: &mut WebsocketContext<WsSession>) {
    if room_name.len() != 2 {
        ctx.text("!!! room name is required");
    } else {
        handler.player.lobby =  room_name[1].to_owned();
        handler.handler.do_send(game::Join { playerdata: handler.player.clone() });
        ctx.text("Joined");
    }
}

fn handle_name_command(handler: &mut WsSession, new_name: Vec<&str>, ctx: &mut WebsocketContext<WsSession>) {
    if new_name.len() != 2 {
        ctx.text("!!! name is required");
    } else {
        handler.player.name = new_name[1].to_owned()
    }
}

fn send_chat_message(handler: &mut WsSession, msg: &str) {
    // Send message to chat server
    handler.handler.do_send(game::ClientMessage {
        player_data: handler.player.clone(),
        msg: msg.to_owned(),
    });
}