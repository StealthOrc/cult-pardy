//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.



use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use actix::{Actor, Addr, Context, Handler, Message, MessageResult, Recipient};
use chrono::TimeDelta;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::TokenResponse;
use rand::random;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use cult_common::{DiscordUser, JeopardyBoard, SessionToken, UserSessionId, WebsocketData};
use cult_common::JeopardyMode::NORMAL;
use crate::authentication::discord::{DiscordME, LoginDiscordAuth};
use crate::servers::authentication::RedeemAdminAccessToken;
use crate::ws::session::UserData;

/// Chat server sends this messages to session
#[derive(Message,Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub enum SessionMessageType {
    Data(WebsocketData),
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

#[derive(Message)]
#[rtype(result = "Vec<String>")]
pub struct ListRooms;

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

#[derive(Message)]
#[rtype(result = "bool")]
pub struct HasLobby {
    pub lobby_id:LobbyId,

}

#[derive(Message)]
#[rtype(result = "UserSession")]
pub struct GetUserSession {
    pub user_session_id: Option<UserSessionId>,
    pub session_token: Option<SessionToken>,
}


#[derive(Message)]
#[rtype(result = "UserSession")]
pub struct JoinWebSocket {
    pub user_session_id: Option<UserSessionId>,
    pub session_token: Option<SessionToken>,
}


#[derive(Message)]
#[rtype(result = "bool")]
pub struct AddDiscordAccount {
    pub user_session_id: UserSessionId,
    pub discord_data: DiscordData,
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
#[derive(Debug)]
pub struct GameServer {
    pub login_discord_auth: LoginDiscordAuth,
    pub rng: ThreadRng,
    pub user_sessions: HashMap<UserSessionId, UserSession>,
    pub lobbies: HashMap<LobbyId, Lobby>
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub struct WebsocketSessionId {
    pub id:usize,
}


impl Serialize for WebsocketSessionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_u64(self.id as u64)
    }
}

impl<'de> Deserialize<'de> for WebsocketSessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let id_str: String = Deserialize::deserialize(deserializer)?;
        let id = id_str.parse().map_err(serde::de::Error::custom)?;
        Ok(WebsocketSessionId { id })
    }
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





#[derive(Debug, Clone,Serialize)]
pub struct UserSession {
    pub user_session_id:UserSessionId,
    pub discord_auth: Option<DiscordData>,
    pub websocket_connections: HashMap<WebsocketSessionId,WebsocketSession>,
    pub session_token: SessionToken,
}




#[derive(Debug, Clone,Serialize)]
pub struct SessionData{
    user_session:UserSession,
    websockets: Vec<WebsocketSessionId>,
}

#[derive(Debug, Clone,Serialize)]
pub struct WebsocketSession{
    websocket_session_id:WebsocketSessionId,
    #[serde(skip_serializing)]
    addr:Recipient<SessionMessageType>,
    lobby_id:LobbyId
}






#[derive(Debug, Clone,Serialize)]
pub struct DiscordData {
    pub(crate) discord_user:Option<DiscordUser>,
    #[serde(skip_serializing)]
    pub(crate) basic_token_response:BasicTokenResponse
}

impl DiscordData {
    async fn update(mut self, basic_token_response:BasicTokenResponse) {
        match DiscordME::get(basic_token_response).await{
            None => self.discord_user = None,
            Some(me) => {
                self.discord_user = Some(me.to_discord_user())
            }
        }
    }

    pub async fn redeem_admin_access_token(self, token:usize) -> Option<RedeemAdminAccessToken>{
        if let Some(discord_user) = self.discord_user {
            return Some(RedeemAdminAccessToken::new(token, discord_user.id))
        } else if let Some(discord_me) = DiscordME::get(self.basic_token_response.clone()).await {
            return Some(RedeemAdminAccessToken::new(token, discord_me.id))
        }
        None
    }
}




impl UserSession {

    pub fn random() -> Self{
        UserSession{
            user_session_id: UserSessionId::random(),
            discord_auth: None,
            websocket_connections: HashMap::new(),
            session_token: SessionToken::random(),
        }
    }

    pub fn to_session_data(self) -> SessionData {
        SessionData{
            user_session: self.clone(),
            websockets: self.websocket_connections.clone().keys().cloned().collect::<Vec<WebsocketSessionId>>(),
        }

    }



    async fn update_discord_auth(mut self, client: BasicClient) {
        if let Some(discord_data) =  self.discord_auth {
            if let Some(token) =  discord_data.basic_token_response.refresh_token() {
                match client.exchange_refresh_token(token).request_async(async_http_client).await{
                    Ok(new_token) => {
                        discord_data.update(new_token).await;
                    },
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



#[derive(Debug, Clone,  Hash, Eq, PartialEq)]
pub struct LobbyId{
    pub id: String,
}

impl Serialize for LobbyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.id)
    }
}

impl<'de> Deserialize<'de> for LobbyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let id: String = Deserialize::deserialize(deserializer)?;
        Ok(LobbyId { id })
    }
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
    pub fn new(login_discord_auth: LoginDiscordAuth) -> GameServer {

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


    fn new_session(&mut self) -> UserSession {
        let mut session= UserSession::random();
        while self.user_sessions.contains_key(&session.user_session_id) {
            session = UserSession::random();
        }
        self.user_sessions.insert(session.user_session_id.clone(), session.clone());
        println!("Added User-session {:?}", session);
        session
    }




    /// Send message to all users in the room
    fn send_lobby_message(&self, lobby_id: &LobbyId, message: WebsocketData) {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
            for user_id in lobby.websocket_session_id.iter() {
                for user_session in self.user_sessions.values() {
                    if let Some(web_socket_session) = user_session.websocket_connections.get(user_id) {
                        web_socket_session.addr.do_send(SessionMessageType::Data(message.clone()));
                    }
                }
            }
        }
    }

    fn send_session_message(&self, lobby_id: &LobbyId, user_session_id: &UserSessionId, message: WebsocketData) {
        if let Some(user_session) = self.user_sessions.get(&user_session_id) {
            for websocket_session in user_session.websocket_connections.values() {
               if websocket_session.lobby_id.eq(lobby_id){
                   websocket_session.addr.do_send(SessionMessageType::Data(message.clone()));
               }
            }
        }
    }
    fn send_websocket_session_message(&self, lobby_id: &LobbyId, websocket_session_id: &WebsocketSessionId,message: WebsocketData) {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
            for user_session_id in  lobby.user_session.clone() {
                if let Some(user_session) = self.user_sessions.get(&user_session_id){
                    if let Some(web_socket_session) = user_session.websocket_connections.get(websocket_session_id) {
                    web_socket_session.addr.do_send(SessionMessageType::Data(message));
                    return;
                }
            }
        }
    }
}




    #[allow(dead_code)]
    fn disconnect(&mut self, user_id:&UserSessionId, lobby_id:&LobbyId) {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
            if let Some(user_session) = lobby.user_session.get(user_id) {
                if let Some(user_session) = self.user_sessions.get(user_session) {
                    for websockets in &lobby.websocket_session_id {
                        if let Some(websocket_session) = user_session.websocket_connections.get(&websockets) {
                            websocket_session.addr.do_send(SessionMessageType::Disconnect);
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
        let user_session = match self.user_sessions.get_mut(&msg.user_session_id) {
            None => return {
                println!("Something happens");
                None
            },
            Some(user_session) => user_session
        };

        let websocket_session_id = WebsocketSessionId::random();

        let web_socket_session = WebsocketSession{
            websocket_session_id,
            addr: msg.addr.clone(),
            lobby_id: msg.lobby_id.clone(),
        };


        &user_session.websocket_connections.insert(websocket_session_id, web_socket_session);


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


        self.send_lobby_message(&msg.lobby_id, WebsocketData::Text("Someone joined".to_string()));
        self.send_websocket_session_message(&msg.lobby_id, &websocket_session_id, WebsocketData::CurrentBoard(JeopardyBoard::default(NORMAL).dto()));
        Some(websocket_session_id)
    }
}

/// Handler for Disconnect message.
impl Handler<SessionDisconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: SessionDisconnect, _: &mut Context<Self>) {
        if let Some(user_session) = self.user_sessions.get_mut(&msg.user_data.user_session_id) {
            if let Some(websocket_session_id) = msg.user_data.websocket_session_id  {
                user_session.websocket_connections.remove(&websocket_session_id);
                if let Some(lobby) = self.lobbies.get_mut(&msg.user_data.lobby) {
                    lobby.websocket_session_id.remove(&websocket_session_id);
                    println!("Someone disconnect: {:?}", user_session.clone().to_session_data());
                    let multi_sessions = user_session.websocket_connections.values().any(|ws | ws.lobby_id.eq(&lobby.lobby_id));
                    if !multi_sessions{
                        lobby.user_session.remove(&msg.user_data.user_session_id);
                    }
                }
            }
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_lobby_message(&msg.player_data.lobby, WebsocketData::Text(msg.msg));
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

impl Handler<GetUserSession> for GameServer {
    type Result = MessageResult<GetUserSession>;

    fn handle(&mut self, msg: GetUserSession, _ctx: &mut Self::Context) -> Self::Result {
        let user_session  = match msg.user_session_id {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
        let token  = match msg.session_token {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
        if let Some(mut session) = self.user_sessions.get_mut(&user_session) {
            if session.clone().session_token.token.eq(&token.token) {
                if token.create.signed_duration_since(session.session_token.create) >TimeDelta::seconds(60){
                    session.session_token.update();
                }
                return return MessageResult(session.clone())
            }
        }
        MessageResult(self.new_session())
    }
}



impl Handler<JoinWebSocket> for GameServer {
    type Result = MessageResult<JoinWebSocket>;

    fn handle(&mut self, msg: JoinWebSocket, _ctx: &mut Self::Context) -> Self::Result {
        let user_session  = match msg.user_session_id {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
        let token  = match msg.session_token {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
        if let Some(mut session) = self.user_sessions.get_mut(&user_session) {
            if session.clone().session_token.token.eq(&token.token) {
                return return MessageResult(session.clone())
            }
        }
        MessageResult(self.new_session())
    }
}


impl Handler<AddDiscordAccount> for GameServer {
    type Result = bool;

    fn handle(&mut self, msg: AddDiscordAccount, ctx: &mut Self::Context) -> Self::Result {
        if let Some(user_session) = self.user_sessions.get_mut(&msg.user_session_id) {
            user_session.discord_auth = Some(msg.discord_data);
            return true
        }
        return false
    }
}


