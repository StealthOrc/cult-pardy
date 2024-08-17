use core::{fmt, hash};
use std::any::{self, Any};
use std::cell::Ref;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use actix::{fut, run, Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, MailboxError, Message, MessageResult, Recipient, WrapFuture};
use actix_web::cookie::time::Date;
use actix_web::rt::{self, task};
use actix_web::web;
use attohttpc::Session;
use chrono::{DateTime, Duration, Local, TimeDelta};

use cult_common::backend::{JeopardyBoard, ActionState, LobbyCreateResponse, MediaPlayer, Question};
use cult_common::dto::board::DTOSession;
use cult_common::wasm_lib::hashs::validate::ValidateHash;
use cult_common::wasm_lib::ids::discord::DiscordID;
use cult_common::wasm_lib::ids::lobby::{self, LobbyId};
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use cult_common::wasm_lib::ids::websocketsession::WebsocketSessionId;
use cult_common::wasm_lib::websocket_events::{ActionMediaEvent, ActionStateEvent, BoardEvent, SessionEvent, VideoEvent, WebsocketError, WebsocketEvent, WebsocketPing, WebsocketServerEvents};
use cult_common::wasm_lib::{DiscordUser, JeopardyMode, Vector2D};
use futures::StreamExt;
use mongodb::bson::{self, doc, Bson, Document};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::TokenResponse;
use rand::distributions::Alphanumeric;
use rand::Rng;
use ritelinked::{LinkedHashMap, LinkedHashSet};
use serde::{Deserialize, Serialize};
use serde_json::{value, Map, Value};
use strum::{Display, EnumIter};
use crate::authentication::discord::DiscordME;
use crate::data::SessionRequest;
use crate::services::authentication::RedeemAdminAccessToken;
use crate::services::StartingServices;
use crate::services::db::DBDatabase::CultPardy;
use crate::services::db::MongoServer;
use crate::ws::session::{self, SendSessionMessageType, UserData};

use super::authentication;
use super::db::UserCollection;
use super::lobby::Lobby;



#[derive(Debug,Clone)]
pub struct LobbyAddrRequest {
    pub lobby_id: LobbyId,
}


pub struct WebsocketConnectionResponse {
    pub websocket_session_id: WebsocketSessionId,
    pub lobby_addr: Lobby,
}


impl Message for LobbyAddrRequest {
    type Result =  Option<Addr<Lobby>>;
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
#[rtype(result = "LobbyCreateResponse")]
pub struct CreateLobby {
    pub user_session_id: UserSessionId,
    pub discord_id: DiscordID,
    pub jeopardy_board:Option<JeopardyBoard>
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
    pub starting_services: Arc<StartingServices>,
    pub lobbies: HashMap<LobbyId, LobbyData>
}



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct UserSession {
    pub user_session_id:UserSessionId,
    pub discord_auth: Option<DiscordData>,
    pub session_token: SessionToken,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct SessionToken {
    pub token: String,
    pub expire: DateTime<Local>,
}

impl Default for SessionToken {
    fn default() -> Self {
        SessionToken {
            token: Self::new_token(),
            expire: (Local::now() + Duration::hours(1)),
        }
    }
}




impl SessionToken {
    pub fn new() -> SessionToken {
        let token = Self::new_token();
        SessionToken {
            token,
            expire: (Local::now() + Duration::hours(1))
        }
    }

    pub fn random() -> SessionToken {
        let token = Self::new_token();
        SessionToken {
            token,
            expire:(Local::now() + Duration::hours(1))
        }
    }

    pub fn update(&mut self) -> SessionToken {
        self.expire = Local::now() + Duration::hours(1);
        self.token = Self::new_token();
        self.clone()
    }

    fn new_token() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect()
    }

    pub fn is_expired(&self) -> bool {
        self.expire < Local::now()
    }


}





#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct FileMetadata {
    pub file_type: String,
    pub validate_hash: ValidateHash,
    pub uploader: DiscordID,
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
        mongo_server.collections.user_sessions.update_one(
            doc! {"user_session_id":user_session_id.id.clone()},
            doc! {"$set": {"discord_auth": {"discord_user": WrappedDiscordUser(discord_user.clone())}}}
        ).await.expect("Cant add the discord Account");
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
    pub fn dto(self, score:&i32, is_admin:bool) -> DTOSession {
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

    pub fn get_discord_id(&self) -> Option<DiscordID> {
        match &self.discord_auth {
            None => None,
            Some(data) => {
                match &data.discord_user {
                    None => None,
                    Some(user) => Some(user.discord_id.clone())
                }
            }
        }
    }

}
#[derive(Debug, Clone)]
pub struct LobbyData {
    pub lobby_id: LobbyId,
    pub addr: Addr<Lobby>,
}


impl GameServer {
    pub fn new(starting_services: Arc<StartingServices>) -> GameServer {
        let name =  LobbyId::from_str("main");

        let lobby = Lobby::new(&starting_services,&name, &UserSessionId::server(), &JeopardyBoard::default(JeopardyMode::NORMAL));
        let addr = lobby.clone().start();

        let lobby_data = LobbyData {
            lobby_id: name.clone(),
            addr,
        };


        let mut lobbies = HashMap::new();
        lobbies.insert(name.clone(), lobby_data);

        println!("Game lobby's: {:?}", &lobbies.values().map(|lobby| lobby.lobby_id.clone()).collect::<Vec<_>>());
        GameServer {
            starting_services,
            lobbies,
        }
    }


    
     async fn get_discord_id(&self, user_session_id: &UserSessionId) -> Option<DiscordID>{
        let user_session = match self.starting_services.mongo_server.find_user_session(&user_session_id).await {
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



    fn new_lobby(&mut self, user_session_id: &UserSessionId,jeopardy_board: &JeopardyBoard) -> LobbyData {
        let mut lobby_id: LobbyId= LobbyId::random();
        while self.lobbies.contains_key(&lobby_id) {
            lobby_id = LobbyId::random();
        }
        let lobby = Lobby::new(&self.starting_services,&lobby_id.clone(), user_session_id, &jeopardy_board);
        let addr  = lobby.start();

        let lobby_data = LobbyData {
            lobby_id: lobby_id.clone(),
            addr,
        };


        self.lobbies.insert(lobby_id.clone(), lobby_data.clone());
        println!("Added Lobby {:?}", lobby_id);
        lobby_data
    }
}


impl Actor for GameServer {
    type Context = Context<Self>;


    fn start(self) -> Addr<Self> where Self: Actor<Context = Context<Self>> {
        Context::new().run(self)
    }
}


impl Handler<LobbyAddrRequest> for GameServer {
    type Result = MessageResult<LobbyAddrRequest>;

    fn handle(&mut self, msg: LobbyAddrRequest, _: &mut Context<Self>) -> Self::Result {
        match self.lobbies.get_mut(&msg.lobby_id) {
                    None => {
                        println!("Lobby not found");
                        MessageResult(None)
                    }
                Some(lobby) => {
                    let addr = lobby.addr.clone();
                    MessageResult(Some(addr))
            }
        }
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


        let board = self.new_lobby(&msg.user_session_id,&board);
        MessageResult(LobbyCreateResponse::Created(board.lobby_id.clone()))
    }
}


pub struct MessageLobbies {
    pub msg: String,
}

impl Message for MessageLobbies {
    type Result = ();
}



impl Handler<MessageLobbies> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: MessageLobbies, _ctx: &mut Self::Context) -> Self::Result {
        for lobby in self.lobbies.values() {
           // lobby.addr.do_send(session::SendSessionMessageType(msg.msg.clone()));
        }
    }
}

