//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use std::{collections::{HashMap, HashSet}, process};
use actix::prelude::*;
use futures::stream::IntoAsyncRead;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::{async_http_client, http_client};
use oauth2::TokenResponse;
use rand::{rngs::ThreadRng, Rng, random};
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::runtime::Runtime;
use cult_common::{UserSessionRequest};
use crate::apis::api::session;
use crate::auth::DiscordME;
use crate::authentication::auth::LoginDiscordAuth;
use crate::ws::custom_ws::GameState as OtherGameState;
use crate::ws::session::UserData;

/// Chat server sends this messages to session
#[derive(Message,Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub enum SessionMessageType {
    MText(String),
    MData(usize),
    Disconnect,
}
#[derive(Debug,Clone)]
pub struct Connect{
    pub lobby_id: LobbyId,
    pub user_session_id:UserSessionId,
    pub addr: Recipient<SessionMessageType>,
}

impl Message for Connect{
    type Result = Option<WebsocketSessionId>;
}

/// Session is disconnected
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct SessionDisconnect{
    pub user_data: UserData,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage{
    pub player_data: UserData,
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
pub struct Join{
    /// Client ID
    pub playerdata: UserData
}

#[derive(Message)]
#[rtype(result = "HashSet<String>")]
pub struct Lobbies;

pub struct HasLobby{
    pub lobby_id:LobbyId,

}
impl actix::Message for HasLobby {
    type Result = bool;
}

pub struct HasUserSession {
    pub user_session_request: Option<UserSessionRequest>,

}
impl actix::Message for HasUserSession {
    type Result = usize;
}

#[allow(dead_code)]
pub struct DiscordAuth {
    pub token: BasicTokenResponse,

}
impl actix::Message for DiscordAuth {
    type Result = DiscordME;
}

#[allow(dead_code)]
pub struct GrandAdminAccess {
    pub discord_id: String,

}
impl actix::Message for GrandAdminAccess {
    type Result = bool;
}





/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
pub struct GameServer{
    login_discord_auth: LoginDiscordAuth,
    rng: ThreadRng,
    user_sessions: HashMap<UserSessionId, UserSession>,
    lobbies: HashMap<LobbyId, Lobby>

}
#[derive(Debug, Clone,Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct UserSessionId {
    pub id:usize,
}

impl UserSessionId{
    pub fn of(id:usize) -> Self{
        UserSessionId{
            id
        }
    }
    pub fn from_string(id:String) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        UserSessionId{
            id
        }
    }
    pub fn from_str(id:&str) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        UserSessionId{
            id
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Copy)]
pub struct WebsocketSessionId {
    pub id:usize,
}
impl WebsocketSessionId{

    pub fn random() ->Self  {
        WebsocketSessionId{
            id: random::<usize>()
        }
    }
    pub fn of(id:usize) -> Self{
        WebsocketSessionId{
            id
        }
    }
    pub fn from_string(id:String) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        WebsocketSessionId{
            id
        }
    }
    pub fn from_str(id:&str) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        WebsocketSessionId{
            id
        }
    }
}


#[derive(Debug, Clone)]
pub struct UserSession {
    user_session_id:UserSessionId,
    discord_auth: Option<BasicTokenResponse>,
    websocket_connections: HashMap<WebsocketSessionId,Recipient<SessionMessageType>>,
}

impl UserSession {
    pub async fn update_discord_auth(mut self, client: BasicClient) {
        if let Some(token) =  self.discord_auth {
            if let Some(token) =  token.refresh_token() {
                match client.exchange_refresh_token(token).request_async(async_http_client).await{
                    Ok(new_token) => self.discord_auth = Some(new_token),
                    Err(error) => {
                        println!("Something happing by requesting new Code {}", error);
                        self.discord_auth = None;
                    }
                }
                return;
            }
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct LobbyId{
    pub id: String,
}
impl LobbyId{
    pub fn of(id:String) -> Self{
        LobbyId{
            id,
        }
    }

    pub fn from_str(id:&str) -> Self{
        LobbyId{
            id:id.to_string(),
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Lobby{
    lobby_id: LobbyId,
    user_session: HashSet<UserSessionId>,
    websocket_session_id: HashSet<WebsocketSessionId>,
    game_state: GameState
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
enum GameState{
    Waiting,
    Starting,
    Playing
}








impl GameServer {
    pub fn new(login_discord_auth: LoginDiscordAuth) -> GameServer{
        // default room
        /*let random_id = rand::thread_rng().sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from).collect();
        */


        let name =  LobbyId::from_str("main");
        let main = Lobby{
            lobby_id: name.clone(),
            user_session: HashSet::new(),
            websocket_session_id: HashSet::new(),
            game_state: GameState::Waiting,
        };


        let mut lobbies = HashMap::new();
        lobbies.insert(name.clone(), main);


        println!("Game lobby's: {:?}", lobbies);
        GameServer {
            login_discord_auth,
            rng: rand::thread_rng(),
            user_sessions: HashMap::new(),
            lobbies,
        }
    }
}
impl GameServer {







    /// Send message to all users in the room
    fn send_message(&self, lobby_id: &LobbyId, message: &str) {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
            for user_id in lobby.websocket_session_id.iter() {
                for user_session in self.user_sessions.values() {
                    if let Some(addr) = user_session.websocket_connections.get(user_id) {
                        addr.do_send(SessionMessageType::MText(message.to_owned()));
                    }
                }
            }
        }
    }
    #[allow(dead_code)]
    fn disconnect(&mut self, user_id:&UserSessionId, lobby_id:&LobbyId) {
        if let Some(mut lobby) = self.lobbies.get(lobby_id) {
            if let Some(user_session) = lobby.user_session.get(user_id) {
                if let Some(user_session) = self.user_sessions.get(user_session) {
                    for websockets in &lobby.websocket_session_id {
                        if let Some(addr) = user_session.websocket_connections.get(&websockets) {
                            addr.do_send(SessionMessageType::Disconnect);
                        }
                    }
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for GameServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;

    fn start(self) -> Addr<Self> where Self: Actor<Context = Context<Self>> {
        Context::new().run(self)
    }
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for GameServer {
    type Result = Option<WebsocketSessionId>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // notify all users in same room
        self.send_message(&msg.lobby_id, "Someone joined");

        println!("->{:?}", msg.user_session_id);
        println!("->{:?}", self.user_sessions);

        let mut user_session = match self.user_sessions.get_mut(&msg.user_session_id) {
            None => return {
                println!("Something happens");
                None
            },
            Some(user_session) => user_session
        };

        let websocket_session_id = WebsocketSessionId::random();

        &user_session.websocket_connections.insert(websocket_session_id, msg.addr.clone());


        match self.lobbies.get_mut(&msg.lobby_id){
            None =>{
                println!("Something happens2");
                user_session.websocket_connections.remove(&websocket_session_id);
                return None
            }
            Some(lobby) => {
                lobby.user_session.insert(msg.clone().user_session_id);
                lobby.websocket_session_id.insert(websocket_session_id);
            }
        }
        println!("Someone joined :  {:?}{:?}", &msg, &websocket_session_id);
        Some(websocket_session_id)
    }
}

/// Handler for Disconnect message.
impl Handler<SessionDisconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: SessionDisconnect, _: &mut Context<Self>) {
        println!("Someone disconnected id: {:?}", msg);
        println!("BEFORE: {:?}", self.user_sessions);
        println!("BEFORE: {:?}", self.lobbies);

        if let Some(mut user_session) = self.user_sessions.get_mut(&msg.user_data.user_session_id) {
            if let Some(websocket_session_id) = msg.user_data.websocket_session_id  {
                if let Some(lobby) = self.lobbies.get_mut(&msg.user_data.lobby) {
                    lobby.websocket_session_id.remove(&websocket_session_id);
                    lobby.user_session.remove(&msg.user_data.user_session_id);
                }
                user_session.websocket_connections.remove(&websocket_session_id);
            }
        }
        println!("AFTER: {:?}", self.user_sessions);
        println!("AFTER: {:?}", self.lobbies);
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.player_data.lobby, msg.msg.as_str());
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for GameServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

       for key in self.lobbies.keys() {
           rooms.push(key.id.to_owned())
       }

        MessageResult(rooms)
    }
}


impl Handler<Lobbies> for GameServer {
    type Result = MessageResult<Lobbies>;

    fn handle(&mut self, _msg: Lobbies, _ctx: &mut Context<Self>) -> Self::Result {
        let mut rooms = HashSet::new();

       for key in self.lobbies.keys() {
           rooms.insert(key.id.to_owned());
       }
        return MessageResult(rooms);
    }
}

impl Handler<HasLobby> for GameServer {
    type Result = bool;

    fn handle(&mut self, msg: HasLobby, _ctx: &mut Context<Self>) -> Self::Result {
        match self.lobbies.get(&msg.lobby_id) {
            None => false,
            Some(sessions) => true
        }
    }
}

impl Handler<HasUserSession> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: HasUserSession, _ctx: &mut Self::Context) -> Self::Result {



        match msg.user_session_request {
            None => {
                let id =  UserSessionId::of(self.rng.gen::<usize>());
                let user_session = UserSession {
                    user_session_id: id.clone(),
                    discord_auth: None,
                    websocket_connections: HashMap::new(),
                };
                self.user_sessions.insert(id.clone(), user_session.clone());
                println!("HIER1{:?}", user_session);
                id.id
            },
            Some(req) => {
                println!("1{:?}", req);

                if let Some(id) = self.user_sessions.get(&UserSessionId::of(req.session_id)) {
                    id.user_session_id.id
                } else {
                    let id =  UserSessionId::of(self.rng.gen::<usize>());
                    let user_session = UserSession {
                        user_session_id: id.clone(),
                        discord_auth: None,
                        websocket_connections: HashMap::new(),
                    };
                    self.user_sessions.insert(id.clone(), user_session.clone());
                    println!("HIER2{:?}", user_session);
                    id.id
                }
            }
        }
    }
}


