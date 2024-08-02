//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

//!!!

use core::fmt;
use std::any::{self, Any};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use actix::{fut, run, Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, MailboxError, Message, MessageResult, Recipient, WrapFuture};
use actix_web::rt::task;
use actix_web::web;
use attohttpc::Session;
use chrono::{DateTime, Duration, Local, TimeDelta};

use cult_common::backend::{JeopardyBoard, LobbyCreateResponse, Question};
use cult_common::dto::DTOSession;
use cult_common::wasm_lib::ids::discord::DiscordID;
use cult_common::wasm_lib::ids::lobby::{self, LobbyId};
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use cult_common::wasm_lib::ids::websocketsession::WebsocketSessionId;
use cult_common::wasm_lib::websocketevents::{BoardEvent, SessionEvent, WebsocketError, WebsocketEvent, WebsocketPing, WebsocketServerEvents};
use cult_common::wasm_lib::{DiscordUser, JeopardyMode, Vector2D};
use futures::StreamExt;
use mongodb::bson::{Bson, doc, Document};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::TokenResponse;
use rand::distributions::Alphanumeric;
use rand::Rng;
use ritelinked::{LinkedHashMap, LinkedHashSet};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use strum::{Display, EnumIter};
use crate::authentication::discord::DiscordME;
use crate::data::SessionRequest;
use crate::servers::authentication::{CheckAdminAccess, GetAdminAccess, RedeemAdminAccessToken};
use crate::servers::StartingServices;
use crate::servers::db::DBDatabase::CultPardy;
use crate::servers::db::MongoServer;
use crate::servers::db::UserCollection::{UserSessions};
use crate::servers::game::GameState::{Playing, Waiting};
use crate::ws::session::{self, UserData};

#[derive(Serialize, Deserialize, Debug)]
pub enum SessionMessageResult {
    Void,
    U64(u64),
}


#[derive(Serialize, Deserialize, Debug)]
pub enum GetSessionMessageType {
    GetPing,
}

/// Chat server sends this messages to session
#[derive(Serialize, Deserialize, Debug)]
pub enum SendSessionMessageType {
    Data(WebsocketServerEvents),
    SelfDisconnect,
}

/// Message for chat server communications
#[derive(Message)]
#[rtype(result = "SessionMessageResult")]
pub enum SessionMessageType {
    Send(SendSessionMessageType),
    Get(GetSessionMessageType),
}


#[derive(Debug,Clone)]
pub struct WebsocketConnect {
    pub lobby_id: LobbyId,
    pub user_session_id:UserSessionId,
    pub addr: Recipient<SessionMessageType>,
    pub ping : i64,
}
#[derive(Message,Debug, Clone)]
#[rtype(result = "()")]
pub struct UpdateWebsocketsPing{
    pub lobby_id: LobbyId,
    pub websocket_session_id: WebsocketSessionId,
    pub ping : i64,
}
#[derive(Debug, Clone)]
pub struct GetSessionsPings{
    pub lobby_id: LobbyId,
}

impl Message for GetSessionsPings {
    type Result = Vec<WebsocketPing>;
    
}

impl Message for WebsocketConnect {
    type Result = Option<WebsocketSessionId>;
}

/// Session is disconnected
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct WebsocketDisconnect {
    pub user_data: UserData,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage{
    pub player_data: UserData,
    pub msg: String,
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
pub struct GetLobbies;

#[derive(Message)]
#[rtype(result = "bool")]
pub struct LobbyExists {
    pub lobby_id:LobbyId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LobbyBackClick {
    pub user_data:UserData,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct LobbyClick {
    pub user_data:UserData,
    pub vector_2d:Vector2D,
}


#[derive(Message)]
#[rtype(result = "bool")]
pub struct CanJoinLobby {
    pub user_session_id:UserSessionId,
    pub lobby_id:LobbyId,
}

#[derive(Message)]
#[rtype(result = "UserSession")]
pub struct GetUserAndUpdateSession {
    pub user_session_id: Option<UserSessionId>,
    pub session_token: Option<SessionToken>,
}

#[derive(Message)]
#[rtype(result = "LobbyCreateResponse")]
pub struct CreateLobby {
    pub user_session_id: UserSessionId,
    pub discord_id: DiscordID,
    pub jeopardy_board:Option<JeopardyBoard>
}



#[derive(Message)]
#[rtype(result = "UserSession")]
pub struct GetUserSession {
    pub user_session_id: Option<UserSessionId>,
    pub session_token: Option<SessionToken>,
}


#[derive(Message)]
#[rtype(result = "DiscordAccountStatus")]
pub struct AddDiscordAccount {
    pub user_session_id: UserSessionId,
    pub discord_data: DiscordData,
}

#[derive(Debug, Clone,PartialEq, EnumIter, Display)]
pub enum DiscordAccountStatus {
    Added,
    Updated(SessionRequest),
    NotAdded,
}

impl DiscordAccountStatus {
    pub fn to_help(self) -> bool {
    match self {
        DiscordAccountStatus::Added => true,
        DiscordAccountStatus::Updated(_) => true,
        DiscordAccountStatus::NotAdded => false,
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddLobbySessionScore {
    pub user_data: UserData,
    pub grant_score_user_session_id: UserSessionId,
    pub vector2d: Vector2D,
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
    pub starting_services: StartingServices,
    pub lobbies: HashMap<LobbyId, Lobby>
}



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct UserSession {
    pub user_session_id:UserSessionId,
    pub discord_auth: Option<DiscordData>,
    pub session_token: SessionToken,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default)]
pub struct SessionToken {
    pub token: String,
    pub create: DateTime<Local>,
}

impl Serialize for SessionToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        let mut doc = Document::new();
        doc.insert("token", self.token.clone());
        doc.insert("create", self.create.clone().to_string());
        doc.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SessionToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        let doc = Document::deserialize(deserializer)?;
        let token = doc.get_str("token").map_err(serde::de::Error::custom)?;
        let create = doc.get_str("create").map_err(serde::de::Error::custom)?;
        let test = DateTime::<Local>::from_str(create).expect("?");
        Ok(SessionToken { token: token.to_string(), create: test})
    }
}


impl From<&SessionToken> for Bson {
    fn from(token: &SessionToken) -> Self {
        let mut doc = Document::new();
        doc.insert("token", token.token.clone());
        doc.insert("create", token.create.clone().to_string());
        Bson::Document(doc)
    }
}


impl SessionToken {
    pub fn new() -> SessionToken {
        let token = Self::new_token();
        SessionToken {
            token,
            create: Local::now(),
        }
    }

    pub fn random() -> SessionToken {
        let token = Self::new_token();
        SessionToken {
            token,
            create: Local::now(),
        }
    }

    pub fn update(&mut self) {
        self.create = Local::now();
        self.token = Self::new_token();
    }
    fn new_token() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect()
    }
}


#[derive(Debug, Clone,Serialize)]
pub struct SessionData{
    user_session:UserSession,
    websockets: Vec<WebsocketSessionId>,
}

#[derive(Debug, Clone, Serialize, Hash, Eq, PartialEq)]
pub struct WebsocketSession{
    websocket_session_id:WebsocketSessionId,
    user_session_id: UserSessionId,
    #[serde(skip_serializing)]
    addr:Recipient<SessionMessageType>,
    ping: i64,
}






#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct DiscordData {
    pub(crate) discord_user:Option<DiscordUser>,
    pub(crate) basic_token_response:BasicTokenResponse
}



pub struct WrappedDiscordUser(pub Option<DiscordUser>);

impl From<WrappedDiscordUser> for Bson {
    fn from(response: WrappedDiscordUser) -> Self {
        match response.0 {
            None => Bson::Document(Document::new()),
            Some(test) => {
                let value: String = serde_json::to_string_pretty(&test).expect("SOME");
                let value: Map<String, Value> = serde_json::from_str(&value).expect("SOME!");
                let doc = Document::try_from(value).expect("??");
                Bson::Document(doc)
            }
        }
    }
}


impl From<DiscordData> for Bson {
    fn from(data: DiscordData) -> Self {
        let mut doc = Document::new();
        doc.insert("discord_user", WrappedDiscordUser(data.discord_user));
        doc.insert("basic_token_response", WrappedBasicTokenResponse(data.basic_token_response));
        Bson::Document(doc)
    }
}


pub struct WrappedBasicTokenResponse(pub BasicTokenResponse);

impl From<WrappedBasicTokenResponse> for Bson {
    fn from(response: WrappedBasicTokenResponse) -> Self {
        let value: String = serde_json::to_string_pretty(&response.0).expect("SOME");
        let value: Map<String, Value> = serde_json::from_str(&value).expect("SOME!");
        let doc = Document::try_from(value).expect("??");
        Bson::Document(doc)
    }
}


impl DiscordData {
    async fn update(mut self, basic_token_response:BasicTokenResponse, mongo_server: &MongoServer, user_session_id: &UserSessionId) {
        let discord_user = match DiscordME::get(basic_token_response).await{
            None =>  None,
            Some(me) => Some(me.to_discord_user())
        };
        mongo_server.collection::<UserSession>(CultPardy(UserSessions)).update_one(
            doc! {"user_session_id":user_session_id.id.clone()},
            doc! {"$set": {"discord_auth": {"discord_user": WrappedDiscordUser(discord_user.clone())}}},
            None,
        ).expect("Cant add the discord Account");
        self.discord_user = discord_user;
    }

    pub(crate) async fn redeem_admin_access_token(self, token:usize) -> Option<RedeemAdminAccessToken>{
        if let Some(discord_user) = self.discord_user {
            return Some(RedeemAdminAccessToken::new(token, discord_user.discord_id))
        } else if let Some(discord_me) = DiscordME::get(self.basic_token_response.clone()).await {
            return Some(RedeemAdminAccessToken::new(token, DiscordID::new(discord_me.id)))
        }
        None
    }
}




impl UserSession {
    fn dto(self, score:&i32, is_admin:bool) -> DTOSession {
        let clone = self.clone();
        let discord_user = match clone.discord_auth {
            None => None,
            Some(data) =>  {
                data.discord_user
            }
        };

        DTOSession{
            user_session_id: clone.user_session_id,
            score:score.clone(),
            discord_user,
            is_admin,
        }
    }

    pub fn random() -> Self {
        UserSession{
            user_session_id: UserSessionId::random(),
            discord_auth: None,
            session_token: SessionToken::random(),
            username: None,
        }
    }


    async fn update_discord_auth(mut self, client: BasicClient, mongo_server: &MongoServer) {
        if let Some(discord_data) =  self.discord_auth {
            if let Some(token) =  discord_data.basic_token_response.refresh_token() {
                match client.exchange_refresh_token(token).request_async(async_http_client).await{
                    Ok(new_token) => {
                        discord_data.update(new_token,mongo_server, &self.user_session_id).await;
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





//TODO ADD CUSTOM Serialize / Deserialize
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Lobby{
    creator: UserSessionId,
    lobby_id: LobbyId,
    user_score: HashMap<UserSessionId, i32>,
    connected_user_session: LinkedHashSet<UserSessionId>,
    allowed_user_session: LinkedHashSet<UserSessionId>,
    websocket_connections: HashMap<WebsocketSessionId,WebsocketSession>,
    game_state: GameState,
    jeopardy_board: JeopardyBoard,
}

impl Lobby{
    pub fn connected_user_score(&self) -> LinkedHashMap<UserSessionId, i32>{
        let mut map = LinkedHashMap::new();
        for session in self.connected_user_session.clone() {
            map.insert(session.clone(), self.user_score.get(&session).unwrap_or(&0).clone());
        }
        map
    }

    pub fn get_session_websockets(&self, user_session_id: &UserSessionId) -> Vec<WebsocketSessionId> {
        self.websocket_connections.values().filter(|websocket_session| websocket_session.user_session_id.eq(user_session_id)).map(|websocket_session| websocket_session.websocket_session_id.clone()).collect()
    }

    pub fn get_session(&self, user_session_id: &UserSessionId) -> Option<UserSession> {
        if self.connected_user_session.contains(user_session_id) {
            return Some(UserSession{
                user_session_id: user_session_id.clone(),
                discord_auth: None,
                session_token: SessionToken::new(),
                username: None,
            });
        }
        None
    }

    pub fn get_session_score(&self, user_session_id: &UserSessionId) -> i32 {
        *self.user_score.get(user_session_id).unwrap_or(&0)
    }

    pub fn update_session_score(&mut self, user_session_id: &UserSessionId, score: i32) {
        let score: i32 = self.user_score.get(user_session_id).unwrap_or(&0) + score;
        self.user_score.insert(user_session_id.clone(), score);
    }

    pub fn update_game_state(&mut self, game_state: GameState) {
        self.game_state = game_state;
    }

    pub fn get_sessions_pings(&self) -> Vec<WebsocketPing> {
        let mut pings = Vec::new();
        for user_session_id in &self.connected_user_session {
            pings.push(WebsocketPing{
                user_session_id: user_session_id.clone(),
                ping: self.get_session_ping(user_session_id),
            });
        }
        pings
    }
    

    pub fn get_session_ping(&self, user_session_id: &UserSessionId) -> i64 {
        let mut ping = 0;
        let mut count = 0;
        for websocket_session in self.get_session_websockets(user_session_id){
            if let Some(websocket_session) = self.websocket_connections.get(&websocket_session){
                let ws_ping = websocket_session.ping;
                ping += ws_ping;
                if ws_ping > 0 {
                    count += 1;
                }
            }
        }
        if count > 0 {
            ping = ping / count;
        } 
        ping
    }
    
    pub fn update_websocket_ping(&mut self, websocket_session_id: &WebsocketSessionId, ping: i64) {
        if let Some(websocket_session) = self.websocket_connections.get_mut(websocket_session_id){
            websocket_session.ping = ping;
        }
    }

    pub fn is_creator(&self, user_session_id: &UserSessionId) -> bool {
        self.creator.eq(user_session_id)
    }

    pub fn set_current_question(&mut self, vector2d: Vector2D) -> Option<Question>{
        let qeuestion = self.jeopardy_board.get_mut_question(vector2d).cloned();
        if qeuestion.is_some() {
            self.jeopardy_board.current = Some(vector2d);
        }
         qeuestion
    }



    pub fn current_question_won(&mut self, grant_score_user_session_id: &UserSessionId) {
        if let Some(value) = self.jeopardy_board.get_value_and_remove_current(&grant_score_user_session_id){
            self.update_session_score(&grant_score_user_session_id, value);
        }
        
    }


    pub fn is_new_session(&self, user_session_id: &UserSessionId) -> bool {
       !self.websocket_connections.values().any(|websocket_session| websocket_session.user_session_id.eq(&user_session_id))
    }

    pub fn is_multiple_session(&self, user_session_id: &UserSessionId) -> bool {
        self.websocket_connections.values().any(|websocket_session| websocket_session.user_session_id.eq(&user_session_id))
    }

    pub fn add_new_websocket(&mut self, websocket_connect: &WebsocketConnect ) -> WebsocketSessionId {
        let websocket_session_id = WebsocketSessionId::random();
        self.websocket_connections.insert(websocket_session_id.clone(), WebsocketSession {
            websocket_session_id:websocket_session_id.clone(),
            user_session_id: websocket_connect.user_session_id.clone(),
            addr: websocket_connect.addr.clone(),
            ping: websocket_connect.ping,
        });
        websocket_session_id
    }

    pub fn add_new_session(&mut self, user_session_id: &UserSessionId, lobby_id:&LobbyId) -> bool{
        self.connected_user_session.insert(user_session_id.clone());
        let allowed_user = self.allowed_user_session.contains(user_session_id);
        // ! maybe need to fix here whenn the user is not allowed
        if !allowed_user {
            println!("New session {:?} has been added to the lobby={:?}.",user_session_id.id, &lobby_id.id);
            self.allowed_user_session.insert(user_session_id.clone());
            self.user_score.insert(user_session_id.clone(), 0);
            return false;
        } else {
            println!("Session {:?} reconnected to the lobby={:?}.",user_session_id.id, &lobby_id.id);
        }

        return true;
    }


    

}



#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, EnumIter, Display)]
pub enum GameState{
    Waiting,
    Starting,
    Playing,
    End
}

impl GameState{

    pub fn open(&self) -> bool {
        match self {
            Waiting => true,
            GameState::Starting => false,
            Playing => false,
            GameState::End => true,
        }



    }


}



impl GameServer {
    pub fn new(starting_services: StartingServices) -> GameServer {
        let name =  LobbyId::from_str("main");
        let main: Lobby = Lobby{
            creator: UserSessionId::server(),
            lobby_id: name.clone(),
            user_score: HashMap::new(),
            connected_user_session: LinkedHashSet::new(),
            allowed_user_session: LinkedHashSet::new(),
            game_state: GameState::Waiting,
            jeopardy_board: JeopardyBoard::default(JeopardyMode::NORMAL),
            websocket_connections: HashMap::new(),
        };


        let mut lobbies = HashMap::new();
        lobbies.insert(name.clone(), main);

        println!("Game lobby's: {:?}", &lobbies.values().map(|lobby| lobby.lobby_id.clone()).collect::<Vec<_>>());
        GameServer {
            starting_services,
            lobbies,
        }
    }

    fn send_someone_joined(&mut self, user_session: UserSession,user_score:i32, lobby_id: &LobbyId) {
        let is_admin = self.is_admin_with_session_id(&user_session.user_session_id);
        self.send_lobby_message(&lobby_id, WebsocketServerEvents::Session(SessionEvent::SessionJoined(user_session.clone().dto(&user_score, is_admin))));
    }

    fn send_current_sessions_to_websocket(&mut self, websocket_session_id: WebsocketSessionId,lobby_id: LobbyId){
        let admins = self.starting_services.mongo_server.get_admins().into_iter().map(|admin| admin.discord_id).collect();
        let msg = WebsocketServerEvents::Session(SessionEvent::CurrentSessions(Vec::from_iter(self.get_dto_sessions(&lobby_id, admins))));
        self.send_websocket_session_message(&lobby_id, &websocket_session_id, msg);
    }

    fn send_current_sessions(&mut self,lobby_id: LobbyId){
        let admins = self.starting_services.mongo_server.get_admins().into_iter().map(|admin| admin.discord_id).collect();
        let msg = WebsocketServerEvents::Session(SessionEvent::CurrentSessions(Vec::from_iter(self.get_dto_sessions(&lobby_id, admins))));
        self.send_lobby_message(&lobby_id, msg);
    }

    


    fn get_discord_id(&self, user_session_id: &UserSessionId) -> Option<DiscordID>{
        let user_session = match self.starting_services.mongo_server.find_user_session(&user_session_id) {
            None => return None,
            Some(data) => data,
        };
        let discord_data = match &user_session.discord_auth {
            None => return None,
            Some(data) => data,
        };
        let discord_user = match &discord_data.discord_user {
            None => return None,
            Some(data) => data,
        };
        Some(discord_user.discord_id.clone())
    }

    #[allow(dead_code)]
    fn get_lobby(&self, lobby_id: &LobbyId) -> Option<&Lobby> {
        self.lobbies.get(lobby_id)
    }

    #[allow(dead_code)]
    fn get_mut_lobby(&mut self, lobby_id: &LobbyId) -> Option<&mut Lobby> {
        self.lobbies.get_mut(lobby_id)
    }

    #[allow(dead_code)]
    fn get_user_session_from_db(&self, user_session_id: &UserSessionId) -> Option<UserSession> {
        return self.starting_services.mongo_server.find_user_session(user_session_id);
    }

    #[allow(dead_code)]
    fn get_user_session_from_lobby(&self, lobby_id: &LobbyId, user_session_id: &UserSessionId) -> Option<UserSession> {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
            if let Some(user_session) = lobby.get_session(user_session_id) {
                return Some(user_session);
            }
        }
        None
    }


    fn new_session(&mut self) -> UserSession {
        let mut session= UserSession::random();
        while  self.starting_services.mongo_server.has_user_session(&session.user_session_id) {
            session = UserSession::random();
        }
        println!("Added User-session {:?}", session.clone());
        self.starting_services.mongo_server.collection::<UserSession>(CultPardy(UserSessions)).insert_one(&session, None).expect("Session not uploaded");
        session
    }

    fn new_lobby(&mut self, user_session_id: UserSessionId,jeopardy_board: JeopardyBoard) -> Lobby {
        let mut lobby_id= LobbyId::random();
        while self.lobbies.contains_key(&lobby_id) {
            lobby_id = LobbyId::random();
        }
        let lobby = Lobby{
            creator: user_session_id,
            lobby_id:lobby_id.clone(),
            user_score: HashMap::new(),
            connected_user_session: LinkedHashSet::new(),
            allowed_user_session: LinkedHashSet::new(),
            websocket_connections: HashMap::new(),
            game_state: GameState::Waiting,
            jeopardy_board,
        };
        self.lobbies.insert(lobby_id.clone(), lobby.clone());
        println!("Added Lobby {:?}", lobby_id);
        lobby
    }

    fn get_dto_sessions(&self, lobby_id: &LobbyId, admin_ids:Vec<DiscordID>) -> LinkedHashSet<DTOSession> {
        let mut sessions = LinkedHashSet::new();
        if let Some(lobby) = self.lobbies.get(&lobby_id){
            for session_id in &lobby.connected_user_session {
                if let Some(session) = self.starting_services.mongo_server.find_user_session(&session_id){
                    let admin = match self.get_discord_id(&session.user_session_id){
                        None => false,
                        Some(discord_id) => admin_ids.contains(&discord_id)
                    };
                    sessions.insert(session.clone().dto(lobby.user_score.get(&session_id).unwrap_or(&0), admin));
                }
            }
        }
        sessions
    }

    // return as sn
    fn is_admin_with_discord_id(&self, discord_id: &DiscordID) -> bool{
        self.starting_services.mongo_server.is_admin(discord_id)
    }

    fn is_admin_with_session_id(&self, user_session_id:&UserSessionId) -> bool{
        if let Some(session) = self.get_discord_id(user_session_id){
            return self.is_admin_with_discord_id(&session);
        }
        false
    }

    pub fn is_editor(&mut self, lobby_id: &LobbyId, user_session_id: &UserSessionId) -> bool {
        let lobby = match self.get_lobby(&lobby_id){
            None => return false,
            Some(lobby) => lobby
        };
        lobby.is_creator(&user_session_id) || self.is_admin_with_session_id(&user_session_id)
    }

    /// Send message to all users in the room
    ///
    fn send_lobby_message(&self, lobby_id: &LobbyId, message: WebsocketServerEvents) {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
            for websocket_session in lobby.websocket_connections.values() {
                Self::send_message(&websocket_session.addr, message.clone());
            }
        }
    }

    pub fn send_message(addr:&Recipient<SessionMessageType>, message: WebsocketServerEvents) {
        addr.do_send(SessionMessageType::Send(SendSessionMessageType::Data(message)));
    }
    
    fn send_websocket_session_message(&self, lobby_id: &LobbyId, websocket_session_id: &WebsocketSessionId,message: WebsocketServerEvents) {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
            if let Some(websocket_session) = lobby.websocket_connections.get(websocket_session_id) {
                websocket_session.addr.do_send(SessionMessageType::Send(SendSessionMessageType::Data(message)));
            }
        }

    }
    #[allow(dead_code)]
    fn send_lobby_session_message(&self, lobby_id: &LobbyId, user_session_id: &UserSessionId, message: WebsocketServerEvents) {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
            let websockets_session = lobby.get_session_websockets(user_session_id);
            for websocket_session_id in websockets_session {
                self.send_websocket_session_message(lobby_id, &websocket_session_id, message.clone());
            }
        }
    }





    #[allow(dead_code)]
    fn disconnect(&mut self, user_id:&UserSessionId, lobby_id:&LobbyId) {
        if let Some(lobby) = self.lobbies.get(lobby_id) {
           for websocket_session in lobby.websocket_connections.values(){
               if websocket_session.user_session_id.eq(user_id){
                   websocket_session.addr.do_send(SessionMessageType::Send(SendSessionMessageType::SelfDisconnect));
                }
            }
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;

    fn start(self) -> Addr<Self> where Self: Actor<Context = Context<Self>> {
        Context::new().run(self)
    }
}


impl Handler<WebsocketConnect> for GameServer {
    type Result = Option<WebsocketSessionId>;

    fn handle(&mut self, msg: WebsocketConnect, _: &mut Context<Self>) -> Self::Result {

        let user_session = match self.get_user_session_from_db(&msg.user_session_id) {
            None => return None,
            Some(data) => data,
        };
        

        let lobby = match self.lobbies.get_mut(&msg.lobby_id) {
            None => {
                println!("Lobby not found");
                Self::send_message(&msg.addr, WebsocketServerEvents::Error(WebsocketError::LobbyNotFound(msg.lobby_id)));
                return None;
            }
            Some(lobby) => lobby,
        };
        
        let new_session = lobby.is_new_session(&msg.user_session_id);
        let websockets = lobby.get_session_websockets(&msg.user_session_id);
        if !new_session && websockets.len() > 1 {
            println!("2 Session {:?} has been already connected to the lobby={:?}.", msg.user_session_id.id, &msg.lobby_id.id);
            return None;
        }




        let websocket_session_id = lobby.add_new_websocket(&msg);



        let creator = lobby.creator.clone();
        
        let board = lobby.jeopardy_board.clone();

        if new_session {
            lobby.add_new_session(&msg.user_session_id, &msg.lobby_id);
            let user_score = *lobby.user_score.get(&msg.user_session_id).unwrap_or(&0);
            let _ = lobby;
            self.send_someone_joined(user_session, user_score, &msg.lobby_id);
        } else {
            println!("WS session has been added to the lobby={:?} for session {:?}.", &msg.lobby_id.id, msg.user_session_id.id);
        }


        let lobby_id = msg.lobby_id.clone();

        self.send_lobby_message(&lobby_id, WebsocketServerEvents::Websocket(WebsocketEvent::WebsocketJoined(websocket_session_id.clone())));
        self.send_websocket_session_message(&lobby_id, &websocket_session_id, WebsocketServerEvents::Board(BoardEvent::CurrentBoard(board.dto(creator))));
        self.send_current_sessions_to_websocket(websocket_session_id.clone(), lobby_id);
        Some(websocket_session_id)
    }
}

/// Handler for Disconnect message.
impl Handler<WebsocketDisconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: WebsocketDisconnect, _: &mut Context<Self>) {
        let _ = match self.starting_services.mongo_server.find_user_session(&msg.user_data.user_session_id) {
            None => return,
            Some(data) => data,
        };
        let websocket_session_id =  match msg.user_data.websocket_session_id {
            None => return,
            Some(websocket_session_id) => websocket_session_id
        };

        let lobby =  match self.get_mut_lobby(&msg.user_data.lobby_id) {
            None => return,
            Some(lobby) => lobby
        };
        
        lobby.websocket_connections.remove(&websocket_session_id);

        let multi_sessions = lobby.is_multiple_session(&msg.user_data.user_session_id);
        if multi_sessions {
            println!("WS session has been removed from the lobby={:?} for session {:?}.", &lobby.lobby_id.id, msg.user_data.user_session_id.id);
        } 

        if !multi_sessions {
            let user_session = msg.user_data.user_session_id.clone();
                lobby.connected_user_session.remove(&user_session);

            // ! NEED TO BE REMOVED AFTER GAME CAN SWITCH TO OTHER STATES

            if !lobby.game_state.open() {

                println!("Session {:?} has been removed from the lobby={:?}.", msg.user_data.user_session_id.id, &lobby.lobby_id.id);
                lobby.allowed_user_session.remove(&user_session);
                lobby.user_score.remove(&user_session);
            } else {
                println!("Session {:?} has been disconnected from the lobby={:?}.", msg.user_data.user_session_id.id, &lobby.lobby_id.id);
            }
        }
        self.send_lobby_message(&msg.user_data.lobby_id, WebsocketServerEvents::Websocket(WebsocketEvent::WebsocketDisconnected(websocket_session_id.clone())));
        if !multi_sessions {
            self.send_lobby_message(&msg.user_data.lobby_id, WebsocketServerEvents::Session(SessionEvent::SessionDisconnected(msg.user_data.user_session_id.clone())));
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_lobby_message(&msg.player_data.lobby_id, WebsocketServerEvents::Text(msg.msg));
    }
}



impl Handler<GetLobbies> for GameServer {
    type Result = MessageResult<GetLobbies>;

    fn handle(&mut self, _msg: GetLobbies, _ctx: &mut Context<Self>) -> Self::Result {
        let mut rooms = HashSet::new();

        for key in self.lobbies.keys() {
            rooms.insert(key.id.to_owned());
        }
        return MessageResult(rooms);
    }
}

impl Handler<LobbyExists> for GameServer {
    type Result = bool;

    fn handle(&mut self, msg: LobbyExists, _ctx: &mut Context<Self>) -> Self::Result {
        match self.lobbies.get(&msg.lobby_id) {
            None => false,
            Some(_sessions) => true
        }
    }
}

impl Handler<GetUserAndUpdateSession> for GameServer {
    type Result = MessageResult<GetUserAndUpdateSession>;

    fn handle(&mut self, msg: GetUserAndUpdateSession, _ctx: &mut Self::Context) -> Self::Result {
        let user_session  = match msg.user_session_id {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
        let token  = match msg.session_token {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };

        let mut user_session = match self.starting_services.mongo_server.find_user_session(&user_session) {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
         if user_session.clone().session_token.token.eq(&token.token) {
            if token.create.signed_duration_since(user_session.session_token.create) >TimeDelta::seconds(60*5){
                user_session.session_token.update(); 
                self.starting_services.mongo_server.collection::<UserSession>(CultPardy(UserSessions)).update_one(
                    doc! {"user_session_id.id": user_session.user_session_id.id.clone()},
                    doc! {"$set": {"session_token": &user_session.session_token}},
                    None,
                ).expect("Cant update User");
            }
            return MessageResult(user_session.clone())
            
        } else{
             println!("Session token not eq user_token:{:?}={:?}", user_session.clone().session_token.token,&token.token )
         }
        MessageResult(self.new_session())
    }
}



impl Handler<GetUserSession> for GameServer {
    type Result = MessageResult<GetUserSession>;

    fn handle(&mut self, msg: GetUserSession, _ctx: &mut Self::Context) -> Self::Result {
        let found_session  = match msg.user_session_id {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
        let token  = match msg.session_token {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };

        let user_session = match self.starting_services.mongo_server.find_user_session(&found_session) {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };

        if user_session.clone().session_token.token.eq(&token.token) {
                return MessageResult(user_session.clone())
        }
        MessageResult(self.new_session())
    }
}


impl Handler<AddDiscordAccount> for GameServer {
    type Result = MessageResult<AddDiscordAccount>;

    fn handle(&mut self, msg: AddDiscordAccount, _ctx: &mut Self::Context) -> Self::Result {
        let status : DiscordAccountStatus;

        let mut user_session = match self.starting_services.mongo_server.find_user_session_with_discord(&msg.discord_data) {
            None => 
                match self.starting_services.mongo_server.find_user_session(&msg.user_session_id) {
                    None => return MessageResult(DiscordAccountStatus::NotAdded),
                    Some(data) => {
                        status = DiscordAccountStatus::Added;
                        data
                    }
                },
            Some(data) => {
                status = DiscordAccountStatus::Updated(SessionRequest{
                    user_session_id: data.user_session_id.clone(),
                    session_token:data.session_token.clone(),
                });
                data
            }
        };
        user_session.discord_auth = Some(msg.discord_data);
        self.starting_services.mongo_server.collection::<UserSession>(CultPardy(UserSessions)).update_one(
            doc! {"user_session_id.id": user_session.user_session_id.id},
            doc! {"$set": {"discord_auth": user_session.discord_auth}},
            None,
        ).expect("Cant add the discord Account");
        return MessageResult(status)
    }
}

impl Handler<CreateLobby> for GameServer {
    type Result = MessageResult<CreateLobby>;

    fn handle(&mut self, msg: CreateLobby, _ctx: &mut Self::Context) -> Self::Result {
        let board = match msg.jeopardy_board {
            None => return MessageResult(LobbyCreateResponse::Error("No JeopardyBoard".to_string())),
            Some(board) => board,
        };
        if board.categories.len() <1 {
            return MessageResult(LobbyCreateResponse::Error("No categories".to_string()))
        }
        let board = self.new_lobby(msg.user_session_id, board);
        MessageResult(LobbyCreateResponse::Created(board.lobby_id))
    }
}

impl Handler<CanJoinLobby> for GameServer {
    type Result = bool;

    fn handle(&mut self, msg: CanJoinLobby, _ctx: &mut Self::Context) -> Self::Result {
        let lobby = match self.lobbies.get(&msg.lobby_id) {
            None => return false,
            Some(lobby) => lobby
        };
        if lobby.game_state.open(){
            return true
        }
        lobby.allowed_user_session.contains(&msg.user_session_id)
    }
}

impl Handler<LobbyClick> for GameServer {
    type Result =  ();

    fn handle(&mut self, msg: LobbyClick, _ctx: &mut Self::Context) -> Self::Result {
        if !self.is_editor(&msg.user_data.lobby_id, &msg.user_data.user_session_id){
            return;
        }
        let lobby = match self.get_mut_lobby(&msg.user_data.lobby_id){
            None => return,
            Some(lobby) => lobby
        };
        if let Some(question) = lobby.set_current_question(msg.vector_2d){
            let event = WebsocketServerEvents::Board(BoardEvent::CurrentQuestion(question.dto(true, msg.vector_2d)));
            self.send_lobby_message(&msg.user_data.lobby_id, event);
        }
    }
}

impl Handler<LobbyBackClick> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: LobbyBackClick, _: &mut Self::Context) -> Self::Result {
        if !self.is_editor(&msg.user_data.lobby_id, &msg.user_data.user_session_id){
            return;
        }
        let lobby = match self.get_mut_lobby(&msg.user_data.lobby_id){
            None => return,
            Some(lobby) => lobby
        };

        lobby.jeopardy_board.current = None;
        let board = lobby.jeopardy_board.clone();
        let event = WebsocketServerEvents::Board(BoardEvent::CurrentBoard(board.dto(lobby.creator.clone())));
        self.send_lobby_message(&msg.user_data.lobby_id, event);
    }
}

impl Handler<AddLobbySessionScore> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: AddLobbySessionScore, _: &mut Self::Context) -> Self::Result {
        if !self.is_editor(&msg.user_data.lobby_id, &msg.user_data.user_session_id){
            return;
        }
        let lobby = match self.get_mut_lobby(&msg.user_data.lobby_id){
            None => return,
            Some(lobby) => lobby
        };

        lobby.current_question_won(&msg.grant_score_user_session_id);

        let board = lobby.jeopardy_board.clone();
        let lobby_id = lobby.lobby_id.clone();

        let event = WebsocketServerEvents::Board(BoardEvent::CurrentBoard(board.dto(lobby.creator.clone())));
        self.send_lobby_message(&msg.user_data.lobby_id, event);
        self.send_current_sessions(lobby_id);

                
        return;
    }

}


impl Handler<UpdateWebsocketsPing> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: UpdateWebsocketsPing, _: &mut Self::Context) -> Self::Result {
        if let Some(lobby) = self.get_mut_lobby(&msg.lobby_id) {
            lobby.update_websocket_ping(&msg.websocket_session_id, msg.ping);
        }
    }
}

impl Handler<GetSessionsPings> for GameServer {
    type Result = MessageResult<GetSessionsPings>;

    fn handle(&mut self, msg: GetSessionsPings, _: &mut Self::Context) -> Self::Result {
        if let Some(lobby) = self.lobbies.get(&msg.lobby_id) {
            MessageResult(lobby.get_sessions_pings())
        } else {
            MessageResult(Vec::new())
        }
    }
}