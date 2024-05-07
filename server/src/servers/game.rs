//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.



use std::collections::{HashMap, HashSet};
use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, fut, Handler, Message, MessageResult, Recipient, WrapFuture};
use actix_web::error::ErrorInternalServerError;
use actix_web::rt::System;
use chrono::TimeDelta;
use futures::executor::block_on;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::TokenResponse;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use cult_common::{BoardEvent, DiscordID, DiscordUser, DTOSession, JeopardyBoard, LobbyCreateResponse, LobbyId, SessionEvent, SessionToken, UserSessionId, Vector2D, WebsocketServerEvents, WebsocketSessionId};
use cult_common::BoardEvent::CurrentBoard;
use cult_common::JeopardyMode::NORMAL;
use cult_common::SessionEvent::SessionDisconnected;
use cult_common::WebsocketError::{LobbyNotFound, SessionNotFound};
use cult_common::WebsocketEvent::{WebsocketDisconnected, WebsocketJoined};
use crate::authentication::discord::{DiscordME, LoginDiscordAuth};
use crate::servers::authentication::{AuthenticationServer, CheckAdminAccess, RedeemAdminAccessToken};
use crate::servers::game;
use crate::servers::game::GameState::Waiting;
use crate::ws::session::UserData;

/// Chat server sends this messages to session
#[derive(Message,Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub enum SessionMessageType {
    Data(WebsocketServerEvents),
    SelfDisconnect,
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
pub struct GetUserSession {
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
pub struct HasSessionForWebSocket {
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
    pub authentication_server: Addr<AuthenticationServer>,
    pub rng: ThreadRng,
    pub user_sessions: HashMap<UserSessionId, UserSession>,
    pub lobbies: HashMap<LobbyId, Lobby>
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
            return Some(RedeemAdminAccessToken::new(token, discord_user.discord_id))
        } else if let Some(discord_me) = DiscordME::get(self.basic_token_response.clone()).await {
            return Some(RedeemAdminAccessToken::new(token, DiscordID::new(discord_me.id)))
        }
        None
    }
}




impl UserSession {
    fn dto(self) -> DTOSession {
        let clone = self.clone();
        let discord_user = match clone.discord_auth {
            None => None,
            Some(data) =>  {
                data.discord_user
            }
        };

        DTOSession{
            user_session_id: clone.user_session_id,
            discord_user,
        }
    }

    pub fn random() -> Self {
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






#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Lobby{
    creator: DiscordID,
    lobby_id: LobbyId,
    user_session: HashSet<UserSessionId>,
    websocket_session_id: HashSet<WebsocketSessionId>,
    game_state: GameState,
    jeopardy_board: JeopardyBoard,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, EnumIter, Display)]
pub enum GameState{
    Waiting,
    Starting,
    Playing,
    End
}








impl GameServer {
    pub fn new(login_discord_auth: LoginDiscordAuth, authentication_server: Addr<AuthenticationServer>) -> GameServer {

        let name =  LobbyId::from_str("main");
        let main = Lobby{
            creator: DiscordID::server(),
            lobby_id: name.clone(),
            user_session: HashSet::new(),
            websocket_session_id: HashSet::new(),
            game_state: GameState::Waiting,
            jeopardy_board: JeopardyBoard::default(NORMAL),
        };


        let mut lobbies = HashMap::new();
        lobbies.insert(name.clone(), main);


        println!("Game lobby's: {:?}", &lobbies.values().map(|lobby| lobby.lobby_id.clone()).collect::<Vec<_>>());
        GameServer {
            login_discord_auth,
            authentication_server,
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

    fn new_lobby(&mut self, discord_id: DiscordID,jeopardy_board: JeopardyBoard) -> Lobby {
        let mut lobby_id= LobbyId::random();
        while self.lobbies.contains_key(&lobby_id) {
            lobby_id = LobbyId::random();
        }
        let lobby = Lobby{
            creator: discord_id,
            lobby_id:lobby_id.clone(),
            user_session: HashSet::new(),
            websocket_session_id: HashSet::new(),
            game_state: GameState::Waiting,
            jeopardy_board,
        };
        self.lobbies.insert(lobby_id.clone(), lobby.clone());
        println!("Added Lobby {:?}", lobby_id);
        lobby
    }

    fn get_dto_sessions(&self, lobby_id: LobbyId) -> HashSet<DTOSession> {
        let mut sessions = HashSet::new();
        if let Some(lobby) = self.lobbies.get(&lobby_id){
            for session_id in &lobby.user_session {
                if let Some(session) = self.user_sessions.get(&session_id){
                    sessions.insert(session.clone().dto());
                }
            }
        }
        sessions
        }




    /// Send message to all users in the room
    ///
    fn send_lobby_message(&self, lobby_id: &LobbyId, message: WebsocketServerEvents) {
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

    pub fn send_message(addr:&Recipient<SessionMessageType>, message: WebsocketServerEvents) {
        addr.do_send(SessionMessageType::Data(message));
    }

    fn send_session_message(&self, lobby_id: &LobbyId, user_session_id: &UserSessionId, message: WebsocketServerEvents) {
        if let Some(user_session) = self.user_sessions.get(&user_session_id) {
            for websocket_session in user_session.websocket_connections.values() {
               if websocket_session.lobby_id.eq(lobby_id){
                   websocket_session.addr.do_send(SessionMessageType::Data(message.clone()));
               }
            }
        }
    }
    fn send_websocket_session_message(&self, lobby_id: &LobbyId, websocket_session_id: &WebsocketSessionId,message: WebsocketServerEvents) {
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
                            websocket_session.addr.do_send(SessionMessageType::SelfDisconnect);
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
        // Notify all users in the same room
        let user_session = match self.user_sessions.get_mut(&msg.user_session_id) {
            None => {
                Self::send_message(&msg.addr, WebsocketServerEvents::Error(SessionNotFound(msg.user_session_id)));
                println!("Something happens");
                return None;
            }
            Some(user_session) => user_session,
        };

        let websocket_session_id = WebsocketSessionId::random();

        let lobby = match self.lobbies.get_mut(&msg.lobby_id) {
            None => {
                println!("Something happens2");
                Self::send_message(&msg.addr, WebsocketServerEvents::Error(LobbyNotFound(msg.lobby_id)));
                return None;
            }
            Some(lobby) => lobby,
        };
        lobby.user_session.insert(msg.clone().user_session_id);
        lobby.websocket_session_id.insert(websocket_session_id.clone());

        let lobby = lobby.clone();

        let new_session = !user_session.websocket_connections.values().any(|websocket_session| websocket_session.lobby_id.eq(&msg.lobby_id));

        user_session.websocket_connections.insert(websocket_session_id.clone(), WebsocketSession {
            websocket_session_id:websocket_session_id.clone(),
            addr: msg.addr.clone(),
            lobby_id: msg.lobby_id.clone(),
        });

        if new_session {
            println!("Someone joined: {:?}{:?}", &msg, &websocket_session_id.clone());
            self.send_lobby_message(&msg.lobby_id.clone(), WebsocketServerEvents::Session(SessionEvent::SessionJoined(msg.user_session_id.clone())));
        }
        let _sessions =

        self.send_lobby_message(&msg.lobby_id, WebsocketServerEvents::Websocket(WebsocketJoined(websocket_session_id.clone())));
        self.send_websocket_session_message(&msg.lobby_id, &websocket_session_id, WebsocketServerEvents::Board(CurrentBoard(lobby.jeopardy_board.dto())));


        self.send_lobby_message(&msg.lobby_id.clone(), WebsocketServerEvents::Session(SessionEvent::CurrentSessions(Vec::from_iter(self.get_dto_sessions(msg.lobby_id)))));

        Some(websocket_session_id)
    }
}

/// Handler for Disconnect message.
impl Handler<SessionDisconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: SessionDisconnect, _: &mut Context<Self>) {
        let user_session =  match self.user_sessions.get_mut(&msg.user_data.user_session_id) {
            None => return,
            Some(user_session) => user_session
        };

        let websocket_session_id =  match msg.user_data.websocket_session_id {
            None => return,
            Some(websocket_session_id) => websocket_session_id
        };

        user_session.websocket_connections.remove(&websocket_session_id);

        let lobby =  match self.lobbies.get_mut(&msg.user_data.lobby_id) {
            None => return,
            Some(lobby) => lobby
        };
        println!("Someone disconnect: {:?}", user_session.clone().to_session_data());
        let multi_sessions = !user_session.websocket_connections.values().any(|ws | ws.lobby_id.eq(&msg.user_data.lobby_id));
        if multi_sessions {
            &lobby.user_session.remove(&msg.user_data.user_session_id);
        }
        self.send_lobby_message(&msg.user_data.lobby_id, WebsocketServerEvents::Websocket(WebsocketDisconnected(websocket_session_id.clone())));
        if multi_sessions {
            self.send_lobby_message(&msg.user_data.lobby_id, WebsocketServerEvents::Session(SessionDisconnected(msg.user_data.user_session_id.clone())));
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
            Some(_sessions) => true
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
        if let Some(session) = self.user_sessions.get_mut(&user_session) {
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



impl Handler<HasSessionForWebSocket> for GameServer {
    type Result = MessageResult<HasSessionForWebSocket>;

    fn handle(&mut self, msg: HasSessionForWebSocket, _ctx: &mut Self::Context) -> Self::Result {
        let user_session  = match msg.user_session_id {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
        let token  = match msg.session_token {
            None => return MessageResult(self.new_session()),
            Some(data) => data,
        };
        if let Some(session) = self.user_sessions.get_mut(&user_session) {
            if session.clone().session_token.token.eq(&token.token) {
                return return MessageResult(session.clone())
            }
        }
        MessageResult(self.new_session())
    }
}


impl Handler<AddDiscordAccount> for GameServer {
    type Result = bool;

    fn handle(&mut self, msg: AddDiscordAccount, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(user_session) = self.user_sessions.get_mut(&msg.user_session_id) {
            user_session.discord_auth = Some(msg.discord_data);
            return true
        }
        return false
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
        let board = self.new_lobby(msg.discord_id, board);
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
        if lobby.game_state.eq(&Waiting){
            return true
        }
        lobby.user_session.contains(&msg.user_session_id)
    }
}

impl Handler<LobbyClick> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: LobbyClick, _ctx: &mut Self::Context) -> Self::Result {
        let user_session = match self.user_sessions.get(&msg.user_data.user_session_id) {
            None => return,
            Some(data) => data,
        };
        let discord_data = match &user_session.discord_auth {
            None => return,
            Some(data) => data,
        };
        let discord_user = match &discord_data.discord_user {
            None => return,
            Some(data) => data,
        };
        let id = discord_user.discord_id.clone();

        let fut = self.authentication_server.send(CheckAdminAccess {
            discord_id: discord_user.clone().discord_id,
        })
            .into_actor(self)
            .then(move |is_admin, msg2, ctx| {
                let is_admin = is_admin.unwrap_or(false);
                let mut lobby = match msg2.lobbies.get_mut(&msg.user_data.lobby_id) {
                    None => return fut::ready(()),
                    Some(lobby) => lobby
                };
                if &lobby.creator.eq(&id) | &is_admin {
                    let category = match lobby.jeopardy_board.categories.get(msg.vector_2d.x as usize) {
                        None => return fut::ready(()),
                        Some(data) => data,
                    };
                    let question = match category.questions.get(msg.vector_2d.y as usize) {
                        None => return fut::ready(()),
                        Some(data) => data,
                    };
                    lobby.jeopardy_board.current = Some(msg.vector_2d);
                    let event = WebsocketServerEvents::Board(
                        BoardEvent::CurrentQuestion(
                            msg.vector_2d,
                            question.clone().dto(true),
                        ),
                    );
                    msg2.send_lobby_message(&msg.user_data.lobby_id, event);
                }
                fut::ready(())
            });
            _ctx.wait(fut);
        return;
    }
}

impl Handler<LobbyBackClick> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: LobbyBackClick, _ctx: &mut Self::Context) -> Self::Result {
        let user_session = match self.user_sessions.get(&msg.user_data.user_session_id) {
            None => return,
            Some(data) => data,
        };
        let discord_data = match &user_session.discord_auth {
            None => return,
            Some(data) => data,
        };
        let discord_user = match &discord_data.discord_user {
            None => return,
            Some(data) => data,
        };
        let id = discord_user.discord_id.clone();

        let fut = self.authentication_server.send(CheckAdminAccess {
            discord_id: discord_user.clone().discord_id,
        })
            .into_actor(self)
            .then(move |is_admin, msg2, ctx| {
                let is_admin = is_admin.unwrap_or(false);
                let mut lobby = match msg2.lobbies.get_mut(&msg.user_data.lobby_id) {
                    None => return fut::ready(()),
                    Some(lobby) => lobby
                };
                if &lobby.creator.eq(&id) | &is_admin {
                    lobby.jeopardy_board.current = None;
                    let board = lobby.jeopardy_board.clone();
                    let event = WebsocketServerEvents::Board(CurrentBoard(board.dto()),
                    );
                    msg2.send_lobby_message(&msg.user_data.lobby_id, event);
                }
                fut::ready(())
            });
        _ctx.wait(fut);
        return;
    }
}




