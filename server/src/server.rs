//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use std::{
    collections::{HashMap, HashSet},
};

use actix::prelude::*;
use oauth2::basic::BasicTokenResponse;
use rand::{rngs::ThreadRng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};
use cult_common::{UserSessionRequest};
use crate::auth::DiscordME;
use crate::session::PlayerData;

/// Chat server sends this messages to session
#[derive(Message,Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub enum SessionDataType {
    MText(String),
    MData(usize),
    Disconnect,
}

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<SessionDataType>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub player_data: PlayerData,
    pub msg: String,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub playerdata: PlayerData
}

#[derive(Message)]
#[rtype(result = "HashSet<String>")]
pub struct Lobbies;

pub struct Lobby{
    pub lobby_id: String,

}
impl actix::Message for Lobby {
    type Result = Option<HashSet<usize>>;
}

pub struct UserSession {
    pub user_session_request: Option<UserSessionRequest>,

}
impl actix::Message for UserSession {
    type Result = usize;
}


pub struct DiscordAuth {
    pub token: BasicTokenResponse,

}
impl actix::Message for DiscordAuth {
    type Result = DiscordME;
}




/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct GameServer {
    wb_sessions: HashMap<usize, Recipient<SessionDataType>>,
    lobby: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    user_session: HashSet<usize>,
    discord_auth: HashMap<usize, BasicTokenResponse>
}

impl GameServer {
    pub fn new() -> GameServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("main".to_owned(), HashSet::new());
        let random_id = rand::thread_rng().sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from).collect();;
        rooms.insert(random_id, HashSet::new());
        println!("{:?}", rooms);
        GameServer {
            wb_sessions: HashMap::new(),
            lobby: rooms,
            rng: rand::thread_rng(),
            user_session: HashSet::new(),
            discord_auth: HashMap::new(),
        }
    }
}

impl GameServer {
    /// Send message to all users in the room
    fn send_message(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.lobby.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.wb_sessions.get(id) {
                        addr.do_send(SessionDataType::MText(message.to_owned()));
                    }
                }
            }
        }
    }

    fn disconnect(&mut self, id:usize) {
        match self.wb_sessions.get(&id) {
            Some(addr) => {
                for (_, sessions) in &mut self.lobby {
                    if let Some(_session) = sessions.get(&id) {
                        sessions.remove(&id);
                    }
                }
                addr.do_send(SessionDataType::Disconnect);
            }
            _ => {}
        }

    }
}

/// Make actor from `ChatServer`
impl Actor for GameServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");



        // notify all users in same room
        self.send_message("main", "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.wb_sessions.insert(id, msg.addr.clone());
        msg.addr.do_send(SessionDataType::MData(1));



        // auto join session to main room
        self.lobby.entry("main".to_owned()).or_default().insert(id);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected id: {}", msg.id);

        let mut rooms: Vec<String> = Vec::new();


        // remove address
        if self.wb_sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.lobby {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.player_data.lobby, msg.msg.as_str(), msg.player_data.id.unwrap());
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for GameServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.lobby.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}


impl Handler<Lobbies> for GameServer {
    type Result = MessageResult<Lobbies>;

    fn handle(&mut self, _msg: Lobbies, _ctx: &mut Context<Self>) -> Self::Result {
        let mut rooms = HashSet::new();

        for key in self.lobby.keys() {
            rooms.insert(key.to_owned());
        }
        return MessageResult(rooms);
    }
}

impl Handler<Lobby> for GameServer {
    type Result = Option<HashSet<usize>>;

    fn handle(&mut self, msg: Lobby, _ctx: &mut Context<Self>) -> Self::Result {
        match self.lobby.get(&msg.lobby_id){
            None => None,
            Some(sessions) => Some(sessions.clone())
        }
    }
}

impl Handler<UserSession> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: UserSession, ctx: &mut Self::Context) -> Self::Result {
        match msg.user_session_request {
            None => {
                let id =  self.rng.gen::<usize>();
                self.user_session.insert(id);
                println!("{:?}", self.user_session);
                id
            },
            Some(req) => {
                if let Some(id) = self.user_session.get(&req.session_id) {
                    *id
                } else {
                    let id =  self.rng.gen::<usize>();
                    self.user_session.insert(id);
                    println!("{:?}", self.user_session);
                    id
                }
            }
        }
    }
}




/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let mut rooms = Vec::new();

        let id = msg.playerdata.id.unwrap();





        // remove session from all rooms
        for (n, sessions) in &mut self.lobby {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }

        self.lobby.entry(msg.playerdata.name.clone()).or_default().insert(id);

        self.send_message(&msg.playerdata.name, "Someone connected",id);
    }
}
