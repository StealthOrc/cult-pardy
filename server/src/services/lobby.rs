
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, MessageResult, Recipient, ResponseActFuture, WrapFuture};

use chrono::{DateTime, Local, Utc};
use cult_common::backend::{ActionState, BuzzerState, JeopardyBoard, MediaState, Question};
use cult_common::dto::board::DTOSession;
use cult_common::wasm_lib::ids::discord::DiscordID;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use cult_common::wasm_lib::ids::websocketsession::WebsocketSessionId;
use cult_common::wasm_lib::websocket_events::{ActionMediaEvent, ActionStateEvent, BoardEvent, MediaStatus, SessionEvent, VideoEvent, WebsocketEvent, WebsocketPing, WebsocketServerEvents};
use cult_common::wasm_lib::{MediaToken, MediaType, QuestionType, Vector2D};
use itertools::Itertools;
use mongodb::bson::doc;
use ritelinked::{LinkedHashMap, LinkedHashSet};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use crate::services::StartingServices;
use crate::services::db::MongoServer;
use crate::ws::session::{SendSessionMessageType, UserData};
use super::authentication::Admin;
use super::game::UserSession;

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub player_data: UserData,
    pub msg: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData{
    user_session:UserSession,
    websockets: Vec<WebsocketSessionId>,
}

#[derive(Debug, Clone, Serialize, Hash, Eq, PartialEq)]
pub struct WebsocketSession {
    pub websocket_session_id:WebsocketSessionId,
    pub user_session_id: UserSessionId,
    #[serde(skip_serializing)]
    pub addr:Recipient<SendSessionMessageType>,
    pings: Vec<i64>,
    pub last_ping: i64,
}



impl WebsocketSession {

    pub fn new(websocket_session_id: WebsocketSessionId, user_session_id: UserSessionId, addr: Recipient<SendSessionMessageType>, ping: i64) -> Self {
        let mut pings = Vec::new();
        pings.push(ping);
        WebsocketSession {
            websocket_session_id,
            user_session_id,
            addr,
            pings,
            last_ping : ping,
        }
    }


    pub fn update_pings(&mut self, ping: i64) {
        if self.pings.len() > 5 {
            self.pings.remove(0);                           
        }
        self.last_ping = self.get_ping();
        self.pings.push(ping);
    }

    pub fn get_ping(&self) -> i64 {
        let mut ping = 0;
        let mut count = 0;
        for ws_ping in &self.pings {
            ping += ws_ping;
            if *ws_ping > 0 {
                count += 1;
            }
        }
        if count > 0 {
            ping = ping / count;
        } 
        ping
    }


}


#[derive(Debug, Clone, Serialize, Hash, Eq, PartialEq)]
pub struct UserSessionData {
    pub user_session_id: UserSessionId,
    pub score: i32,
}

impl UserSessionData {
    pub fn default(user_session_id:&UserSessionId) -> Self {
        UserSessionData {
            score: 0,
            user_session_id: user_session_id.clone(),
        }
    }
    
}


//TODO ADD CUSTOM Serialize / Deserialize
#[derive(Debug, Clone)]
pub struct Lobby {
    pub starting_services: Arc<StartingServices>,
    pub lobby_id: LobbyId,
    pub creator: UserSessionId,
    pub user_data: HashMap<UserSessionId, UserSessionData>,
    pub connected_user_session: LinkedHashSet<UserSessionId>,
    pub allowed_user_session: LinkedHashSet<UserSessionId>,
    pub websocket_connections: HashMap<WebsocketSessionId,WebsocketSession>,
    pub game_state: GameState,
    pub jeopardy_board: JeopardyBoard,
}



impl Actor for Lobby {
    type Context = Context<Self>;

    fn start(self) -> Addr<Self> where Self: Actor<Context = Context<Self>> {
        Context::new().run(self)
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Lobby started");
        //self.send_pings(ctx);
    }

}



impl Lobby {

    pub fn new(starting_services:&Arc<StartingServices>, lobby_id: &LobbyId, creator: &UserSessionId, jeopardy_board: &JeopardyBoard) -> Self {
        let mut allowed_user_session: LinkedHashSet<UserSessionId> = LinkedHashSet::new();
        allowed_user_session.insert(creator.clone());
        let game_state = GameState::Waiting;
        Lobby {
            starting_services: starting_services.clone(),
            lobby_id : lobby_id.clone(),
            creator : creator.clone(),
            user_data: HashMap::new(),
            connected_user_session: LinkedHashSet::new(),
            allowed_user_session,
            websocket_connections:HashMap::new(),
            game_state,
            jeopardy_board:jeopardy_board.clone(),
        }
    }




    fn send_pings(&self, ctx: &mut Context<Self>) {
        /*ctx.run_interval(Duration::from_secs(1), |act: &mut Lobby, _| {
            if act.websocket_connections.is_empty() {
                return;
            }
            let session_pings = act.get_sessions_pings();
            for websocket_session in act.websocket_connections.values() {
                let event = SessionEvent::SessionsPing(session_pings.clone());
                websocket_session.addr.do_send(SendSessionMessageType::Data(WebsocketServerEvents::Session(event)));
            }
        });*/ 
    }




    pub fn connected_user_score(&self) -> LinkedHashMap<UserSessionId, i32>{
        let mut map = LinkedHashMap::new();
        for session_id in  self.connected_user_session.clone() {
            let score = self.get_session_score(&session_id);
            map.insert(session_id, score);
        }
        map
    }

    pub fn get_session_websockets(&self, user_session_id: &UserSessionId) -> Vec<WebsocketSessionId> {
        self.websocket_connections.values().filter(|websocket_session| websocket_session.user_session_id.eq(user_session_id)).map(|websocket_session| websocket_session.websocket_session_id.clone()).collect()
    }



    pub fn get_session_score(&self, user_session_id: &UserSessionId) -> i32 {
        if let Some(_) = self.connected_user_session.get(user_session_id) {
            if let Some(user_data) = self.user_data.get(user_session_id) {
                return user_data.score;
                }
            }
        0
    }

    pub fn update_session_score(&mut self, user_session_id: &UserSessionId, score: i32) {
        if let Some(_) = self.connected_user_session.get(user_session_id) {
            if let Some(user_data) = self.user_data.get_mut(user_session_id) {
                user_data.score += score;
                }
            }
    }

    pub fn update_game_state(&mut self, game_state: GameState) {
        self.game_state = game_state;
    }

    pub fn get_sessions_pings(&self) -> Vec<WebsocketPing> {
        let mut pings: Vec<WebsocketPing> = Vec::new();
        for user_session_id in &self.connected_user_session{
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
                let ws_ping = websocket_session.get_ping();
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

    pub fn get_session_last_ping(&self, user_session_id: &UserSessionId) -> i64 {
        let mut ping = 0;
        let mut count = 0;
        for websocket_session in self.get_session_websockets(user_session_id){
            if let Some(websocket_session) = self.websocket_connections.get(&websocket_session){
                let ws_ping = websocket_session.last_ping;
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

    
    pub fn is_creator(&self, user_session_id: &UserSessionId) -> bool {
        self.creator.eq(user_session_id)
    }

    pub fn set_current_question(&mut self, vector2d: Vector2D, websocket_session_id: &WebsocketSessionId) -> Option<Question>{
        let qeuestion = self.jeopardy_board.get_mut_question(vector2d).cloned();
        if let Some(value) = qeuestion.clone() {
            self.jeopardy_board.current = Some(vector2d);
            let mut state = self.jeopardy_board.action_state.lock().expect("Failed to lock action state");
            let action: ActionState = value.question_type.get_default_actionstate(websocket_session_id);
            state.update(action)
        }
         qeuestion
    }



    pub fn current_question_won(&mut self, grant_score_user_session_id: &UserSessionId) {
        if let Some(value) = self.jeopardy_board.get_value_and_remove_current(&grant_score_user_session_id){
            self.update_session_score(&grant_score_user_session_id, value);
        }
        
    }


    pub fn has_session_websockets(&self, user_session_id: &UserSessionId) -> bool {
        self.websocket_connections.values().any(|websocket_session| websocket_session.user_session_id.eq(&user_session_id)
        )
    }

    pub fn is_new_session(&self, user_session_id: &UserSessionId) -> bool {
        !self.connected_user_session.contains(user_session_id)
    }



    pub fn is_multiple_session(&self, user_session_id: &UserSessionId) -> bool {
        self.websocket_connections.values().any(|websocket_session| websocket_session.user_session_id.eq(&user_session_id))
    }

    pub fn add_new_websocket(&mut self, websocket_connect: &WebsocketConnect ) -> WebsocketSessionId {
        let websocket_session_id = WebsocketSessionId::random();
        self.websocket_connections.insert(websocket_session_id.clone(), WebsocketSession::new(websocket_session_id.clone(), websocket_connect.user_session_id.clone(), websocket_connect.addr.clone(), websocket_connect.ping));
        websocket_session_id
    }



    pub fn add_new_session(&mut self, user_session_id: &UserSessionId, user_session_data: &UserSessionData){
        self.connected_user_session.insert(user_session_id.clone()); 
        self.user_data.insert(user_session_id.clone(), user_session_data.clone());
        println!("New session {:?} has been added to the lobby={:?}.",user_session_id.id, &self.lobby_id);
    }

    pub fn reconnect_session(&mut self, user_session_id: &UserSessionId) {
        self.connected_user_session.insert(user_session_id.clone());
        println!("Session {:?} reconnected to the lobby={:?}.",user_session_id.id, &self.lobby_id);
    }

    pub fn send_lobby_message(&self, event: &WebsocketServerEvents) {
        for websocket_session in self.websocket_connections.values() {
            if websocket_session.addr.connected() {
                let _ = websocket_session.addr.do_send(SendSessionMessageType::Data(event.clone()));
            }
        }
    }

    pub fn get_get_user_session_ids(&self) -> Vec<UserSessionId> {
        self.connected_user_session.iter().cloned().collect()
    }




    pub async fn get_dto_sessions(&self) -> Vec<DTOSession> {
        let mut sessions: Vec<DTOSession> = Vec::new();
        let admin = self.get_admins().await;
        for user_session_id in &self.connected_user_session {
            let user_session = match get_session(&self.starting_services.mongo_server, user_session_id).await {
                None => continue,
                Some(session) => session,
            };
            let is_admin = match user_session.get_discord_id() {
                None => false,
                Some(discord_id) => admin.iter().any(|admin| admin.discord_id.eq(&discord_id)),
            };
            let dto = user_session.dto(&self.get_session_score(user_session_id), is_admin);
            sessions.push(dto);
        }
        sessions
    }

    pub async fn send_current_sessions(&self) {
        let session_vec = self.get_dto_sessions().await;
        let event = WebsocketServerEvents::Session(SessionEvent::CurrentSessions(session_vec));
        self.send_lobby_message(&event);
    }


    pub fn get_user_session_data(&self, user_session_id: &UserSessionId) -> UserSessionData  {
        match self.user_data.get(user_session_id) {
            None => return UserSessionData::default(user_session_id),
            Some(data) => data.clone(),
        }
    }




    pub async fn is_editor(&self, user_session: &UserSession) -> bool {
        if self.is_creator(&user_session.user_session_id) {
            return true;
        }
        let discord_id = match self.get_discord_id(&user_session) {
            None => return false,
            Some(data) => data,
        };

        let is_admin = self.starting_services.mongo_server.is_admin(&discord_id).await;
        is_admin
    }

    pub async fn send_someone_joined(&self, user_session:&UserSession, user_score: i32){
        let is_admin = self.is_admin(&user_session).await;
        let dto =  user_session.clone().dto(&user_score, is_admin);
        let event = WebsocketServerEvents::Session(SessionEvent::SessionJoined(dto));
        self.send_lobby_message(&event);
    }

    pub fn get_discord_id(&self,user_session: &UserSession) -> Option<DiscordID>{
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
    pub async fn is_admin(&self, user_session: &UserSession) -> bool {
        let discord_id = match self.get_discord_id(&user_session) {
            None => return false,
            Some(data) => data,
        };
        self.starting_services.mongo_server.is_admin(&discord_id).await
    }

    pub async fn get_admins(&self) -> HashSet<Admin> {
        self.starting_services.mongo_server.get_admins().await
    }

    pub fn send_websocket_session_message(&self,websocket_session_id: &WebsocketSessionId, event: WebsocketServerEvents) {
        if let Some(websocket_session) = self.websocket_connections.get(websocket_session_id) {
            if websocket_session.addr.connected() {
                let _ = websocket_session.addr.do_send(SendSessionMessageType::Data(event));
            }
        }
    }

    pub fn send_websocket_current_session(&self, websocket_session_id: &WebsocketSessionId, event: &WebsocketServerEvents) {
        self.send_websocket_session_message(&websocket_session_id, event.clone());
    }

}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Display)]
pub enum GameState{
    Waiting,
    Starting,
    Playing,
    End
}

impl GameState{

    pub fn open(&self) -> bool {
        match self {
            GameState::Waiting => true,
            GameState::Starting => false,
            GameState::Playing => false,
            GameState::End => true,
        }
    }
}
pub struct LobbyClick {
    pub user_data:UserData,
    pub vector_2d:Vector2D,
}


impl Message for LobbyClick {
    type Result = ();
    
}
pub async fn is_editor(user_session_id: &UserSessionId,creator: &UserSessionId,db: Arc<MongoServer>) -> bool {
        if creator.eq(user_session_id) {
             return true;
        }
        let session = match get_session(&db, user_session_id).await{
            None => return false,
            Some(session) => session
        };

        let discord_id = match session.get_discord_id() {
            None => return false,
            Some(data) => data
        };
        db.is_admin(&discord_id).await

}


impl Handler<LobbyClick> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: LobbyClick, _ctx: &mut Self::Context) -> Self::Result {
        let user_session_id: UserSessionId = msg.user_data.user_session_id.clone();
        let creator: UserSessionId = self.creator.clone();
        let db: Arc<MongoServer> = self.starting_services.mongo_server.clone();
        Box::pin(
            async move {
                is_editor(&user_session_id, &creator, db).await
            }.into_actor(self).map(move |allowed, lobby, _|  {
            if allowed.clone() {
                let ws_id: WebsocketSessionId = match msg.user_data.websocket_session_id.clone() {
                    None => return,
                    Some(id) => id,
                };
                let vec = msg.vector_2d.clone();
                if let Some(question) = lobby.set_current_question(vec,  &ws_id) {
                    let action_state = lobby.jeopardy_board.action_state.lock().expect("Failed to lock action state").clone();
                    let event = WebsocketServerEvents::Board(BoardEvent::CurrentQuestion(question.dto(true, vec.clone()), action_state));
                    lobby.send_lobby_message(&event);
                }
            }
        }))
    }
}



#[derive(Message)]
#[rtype(result = "()")]
pub struct LobbyBackClick {
    pub user_data:UserData,
}

impl Handler<LobbyBackClick> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: LobbyBackClick, _: &mut Self::Context) -> Self::Result {
        let user_session_id = msg.user_data.user_session_id.clone();
        let creator = self.creator.clone();
        let db = self.starting_services.mongo_server.clone();

        Box::pin(
            async move {
                is_editor(&user_session_id, &creator, db).await
        }.into_actor(self).map(move |allowed, lobby, _|  {
            if allowed.clone() {
                lobby.jeopardy_board.current = None;
                if let Ok(mut state) = lobby.jeopardy_board.action_state.lock(){
                    state.update(ActionState::None);
                }
                let board = lobby.jeopardy_board.clone();
                println!("Back Clicked {:#?}", board.action_state);
                let event = WebsocketServerEvents::Board(BoardEvent::CurrentBoard(board.dto(lobby.creator.clone())));
                lobby.send_lobby_message(&event);
            }
        }))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddLobbySessionScore {
    pub user_data: UserData,
    pub grant_score_user_session_id: UserSessionId,
    pub vector2d: Vector2D,
}


impl Handler<AddLobbySessionScore> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: AddLobbySessionScore, _: &mut Self::Context) -> Self::Result {
        let user_session_id = msg.user_data.user_session_id.clone();
        let creator = self.creator.clone();
        let db = self.starting_services.mongo_server.clone();
        Box::pin(
            async move {
                is_editor(&user_session_id, &creator, db).await
        }.into_actor(self).map(move |allowed, lobby,ctx|  {
            if allowed.clone() {
                lobby.current_question_won(&msg.grant_score_user_session_id);
                let dto_board = lobby.jeopardy_board.dto(lobby.creator.clone());
                let event = WebsocketServerEvents::Board(BoardEvent::CurrentBoard(dto_board));
                lobby.send_lobby_message(&event);
                //FIXME: Send the user session id
                ctx.address().do_send(SendCurrentDTOSessions{});
            }
        }))
    }
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct SendCurrentDTOSessions {
}


impl Handler<SendCurrentDTOSessions> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _: SendCurrentDTOSessions, _: &mut Self::Context) -> Self::Result {
        let db: Arc<MongoServer> = self.starting_services.mongo_server.clone();
        let user_session_data = self.user_data.clone();
        let user_session_id = self.connected_user_session.iter().map(|id| id.clone()).collect::<Vec<UserSessionId>>();


        Box::pin(
            async move {
                let sessions = get_sessions(&db, &user_session_id).await;
                let dto_sessions = get_dto_sessions(&db, sessions, user_session_data).await;
                WebsocketServerEvents::Session(SessionEvent::CurrentSessions(dto_sessions))
        }.into_actor(self).map(move |event, lobby, _|  {
            lobby.send_lobby_message(&event);
        }))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendDTOSessionJoined {
    pub user_session_id: UserSessionId,
}


impl Handler<SendDTOSessionJoined> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: SendDTOSessionJoined, _: &mut Self::Context) -> Self::Result {
        let db: Arc<MongoServer> = self.starting_services.mongo_server.clone();
        let user_session_data = self.get_user_session_data(&msg.user_session_id);
        let user_session_id = msg.user_session_id.clone();
        


        Box::pin(
            async move {
                let user_session = match get_session(&db, &user_session_id).await {
                    None => return None,
                    Some(session) => session,
                };
                let dto_session = get_dto_sesion(&db, &user_session, &user_session_data).await;
                Some(WebsocketServerEvents::Session(SessionEvent::SessionJoined(dto_session)))
        }.into_actor(self).map(move |event, lobby, _|  {

            if let Some(event) = event {
                lobby.send_lobby_message(&event);
            }
        }))
    }
}








#[derive(Message,Debug, Clone)]
#[rtype(result = "()")]
pub struct UpdateWebsocketPing{
    pub websocket_session_id: WebsocketSessionId,
    pub ping : i64,
}

impl Handler<UpdateWebsocketPing> for Lobby {
    type Result = ();

   fn handle(&mut self, msg: UpdateWebsocketPing, _: &mut Self::Context) -> Self::Result {
        if let Some(websocket_session) = self.websocket_connections.get_mut(&msg.websocket_session_id) {
            websocket_session.update_pings(msg.ping);
            let user_session_id = websocket_session.user_session_id.clone();
            let ping: i64 = self.get_session_ping(&user_session_id);


            if ping == self.get_session_last_ping(&user_session_id) {
                return;
            }
            let web_socket_ping: WebsocketPing = WebsocketPing {
                user_session_id,
                ping,
            };
            
            

            let event = SessionEvent::SessionPing(web_socket_ping);
            self.send_lobby_message(&WebsocketServerEvents::Session(event))
        }
    }
}



#[derive(Message)]
#[rtype(result = "()")]
pub struct ReciveVideoEvent{
    pub user_session_id: UserSessionId,
    pub websocket_session_id: WebsocketSessionId,
    pub event: VideoEvent,
}
const THRESH_IGNORANCE: f64 = 250.0;

impl Handler<ReciveVideoEvent> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: ReciveVideoEvent, _: &mut Self::Context) -> Self::Result {
        let user_session_id = msg.user_session_id.clone();
        let creator = self.creator.clone();
        let db = self.starting_services.mongo_server.clone();

        Box::pin(
            async move {
                is_editor(&user_session_id, &creator, db).await
        }.into_actor(self).map(move |allowed, lobby: &mut Lobby, _|  {
            let mut state = lobby.jeopardy_board.action_state.lock().expect("Failed to lock action state");
            
            let current_media_state = state.get_media_player_or_default(&msg.websocket_session_id);
            let current_status = &current_media_state.status;

            println!("before Action state {:#?}", lobby.jeopardy_board.action_state);
            
            if allowed.clone() {
                match msg.event {
                    VideoEvent::ChangeState(mut new_media_status) => {
                        new_media_status.interaction_id = msg.websocket_session_id.clone();
                        let to_soon = (Local::now().timestamp_millis() as f64 - current_status.last_updated) < THRESH_IGNORANCE;
                        let other_ws = !current_status.interaction_id.id.eq(&msg.websocket_session_id.id);
                        let stale  = new_media_status.last_updated < current_status.last_updated;

                        if !stale && !(to_soon && other_ws) {
                            println!("Changing state to {:?}", new_media_status.clone());
                            let new_media_state = MediaState {
                                status: new_media_status.clone(),
                                current_media: current_media_state.current_media.clone(),
                            };
                            state.update(ActionState::MediaPlayer(new_media_state));
                            drop(state);
                            println!("after Action state {:#?}", lobby.jeopardy_board.action_state);

                            let event: WebsocketServerEvents = WebsocketServerEvents::ActionState(ActionStateEvent::Media(ActionMediaEvent::ChangeState(new_media_status)));
                            lobby.send_lobby_message(&event);
                        }
                    }
                    //NOT USESD
                    VideoEvent::Play => {
                        let event = WebsocketServerEvents::ActionState(ActionStateEvent::Media(ActionMediaEvent::Play));
                        lobby.send_lobby_message(&event);
                    }
                    //NOT USESD
                    VideoEvent::Pause(_) => {
                        let event = WebsocketServerEvents::ActionState(ActionStateEvent::Media(ActionMediaEvent::Pause));
                        lobby.send_lobby_message(&event);
                    }
                    //NOT USESD
                    VideoEvent::Resume => {
                        let event = WebsocketServerEvents::ActionState(ActionStateEvent::Media(ActionMediaEvent::Resume));
                        lobby.send_lobby_message(&event);
                    }
                }
            }
        }))

    }
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct SyncBackwardRequest{
    pub websocket_session_id: WebsocketSessionId,
}

impl Handler<SyncBackwardRequest> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: SyncBackwardRequest, _: &mut Self::Context) -> Self::Result {
        let current_time = Local::now().timestamp_millis();
        let event = WebsocketServerEvents::ActionState(ActionStateEvent::SyncBackward(current_time));
        self.send_websocket_session_message(&msg.websocket_session_id, event);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SyncForwardRequest{
    pub websocket_session_id: WebsocketSessionId,
    pub current_time: f64,
}

impl Handler<SyncForwardRequest> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: SyncForwardRequest, _: &mut Self::Context) -> Self::Result {
        let current_time =  Local::now().timestamp_millis() as f64 - msg.current_time;
        let event = WebsocketServerEvents::ActionState(ActionStateEvent::SyncForward(current_time));
        self.send_websocket_session_message(&msg.websocket_session_id, event)
    }
}







pub struct WebsocketConnect {
    pub user_session_id: UserSessionId,
    pub addr: Recipient<SendSessionMessageType>,
    pub ping: i64,
}


impl Message for WebsocketConnect {
    type Result = Option<WebsocketSessionId>;
    
}


struct WebsocketConnectFuture{
    pub dto_session: Option<DTOSession>,
    pub dto_sessions: Vec<DTOSession>,

}



impl Handler<WebsocketConnect> for Lobby {
    type Result =    Option<WebsocketSessionId>;
    fn handle(&mut self, msg: WebsocketConnect, ctx: &mut Context<Self>) -> Self::Result {

        let has_websockets: bool = self.has_session_websockets(&msg.user_session_id);
        let websockets = self.get_session_websockets(&msg.user_session_id);

        if has_websockets && websockets.len() > 1 {
            println!("2 Session {:?} has been already connected to the lobby={:?}.", msg.user_session_id.id, &self.lobby_id.id);
            return None;
        } 


        let user_session_data = self.get_user_session_data(&msg.user_session_id);

        let websocket_session_id = self.add_new_websocket(&msg);
        let event = WebsocketServerEvents::Websocket(WebsocketEvent::WebsocketJoined(websocket_session_id.clone()));
        self.send_lobby_message(&event);


        let is_new_session = self.is_new_session(&msg.user_session_id);
        if is_new_session {
            self.add_new_session(&msg.user_session_id, &user_session_data);
        } else {
            self.reconnect_session(&msg.user_session_id);
        }

        ctx.address().do_send(SendWSCurrentDTOBoard{websocket_session_id: websocket_session_id.clone()});
        ctx.address().do_send(SendDTOSessionJoined{user_session_id: msg.user_session_id.clone()});
        ctx.address().do_send(SendCurrentDTOSessions{}); 
        


        let session_pings = self.get_sessions_pings();

        let event = WebsocketServerEvents::Session(SessionEvent::SessionsPing(session_pings));
        self.send_websocket_session_message(&websocket_session_id, event);
        return Some(websocket_session_id);
        

    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct SendWSCurrentDTOBoard {
    pub websocket_session_id: WebsocketSessionId,
}


impl Handler<SendWSCurrentDTOBoard> for Lobby {
    type Result =  ();
    fn handle(&mut self, msg: SendWSCurrentDTOBoard, _: &mut Context<Self>) -> Self::Result {
        println!("Action state {:#?}", self.jeopardy_board.action_state);
        let dto_board = self.jeopardy_board.dto(self.creator.clone());
        let event = WebsocketServerEvents::Board(BoardEvent::CurrentBoard(dto_board.clone()));
        self.send_websocket_session_message(&msg.websocket_session_id, event);
            
    }
}







/// Session is disconnected
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct WebsocketDisconnect {
    pub user_data: UserData,
}

impl Handler<WebsocketDisconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: WebsocketDisconnect, _: &mut Context<Self>) {
        let websocket_session_id =  match msg.user_data.websocket_session_id {
            None => return,
            Some(websocket_session_id) => websocket_session_id
        };

        self.websocket_connections.remove(&websocket_session_id);

        let multi_sessions = self.is_multiple_session(&msg.user_data.user_session_id);
        if multi_sessions {
            println!("WS session has been removed from the lobby={:?} for session {:?}.", &self.lobby_id.id, msg.user_data.user_session_id.id);
        } 

        if !multi_sessions {
            let user_session = msg.user_data.user_session_id.clone();
            self.connected_user_session.remove(&user_session);

            // ! NEED TO BE REMOVED AFTER GAME CAN SWITCH TO OTHER STATES

            if !self.game_state.open() {
                println!("Session {:?} has been removed from the lobby={:?}.", msg.user_data.user_session_id.id, &self.lobby_id.id);
                self.allowed_user_session.remove(&user_session);
            } else {
                println!("Session {:?} has been disconnected from the lobby={:?}.", msg.user_data.user_session_id.id, &self.lobby_id.id);
            }
        }
        self.send_lobby_message( &WebsocketServerEvents::Websocket(WebsocketEvent::WebsocketDisconnected(websocket_session_id.clone())));
        if !multi_sessions {
            self.send_lobby_message( &WebsocketServerEvents::Session(SessionEvent::SessionDisconnected(msg.user_data.user_session_id.clone())));
        }
    }


}



#[derive(Message)]
#[rtype(result = "bool")]
pub struct CanJoinLobby {
    pub user_session_id:UserSessionId,
}



impl Handler<CanJoinLobby> for Lobby {
    type Result = bool;

    fn handle(&mut self, msg: CanJoinLobby, _ctx: &mut Self::Context) -> Self::Result {

        if self.game_state.open() {
            return true
        }
        self.allowed_user_session.contains(&msg.user_session_id)
    }
}


#[derive(Message)]
#[rtype(result = "Option<MediaToken>")]
pub struct GetMediaToken;


impl Handler<GetMediaToken> for Lobby {
    type Result = MessageResult<GetMediaToken>;

    fn handle(&mut self, _: GetMediaToken, _: &mut Self::Context) -> Self::Result {
        let current = match self.jeopardy_board.current.clone() {
            Some(c) => c,
            None => return MessageResult(None),
        };

        let question = match self.jeopardy_board.get_question(current) {
            Some(q) => q,
            None => return MessageResult(None),
        };

        let state = match self.jeopardy_board.action_state.lock() {
            Ok(s) => s,
            Err(_) => return MessageResult(None), 
        };

        let media_player = match state.get_media_player() {
            Some(mp) => mp,
            None => return MessageResult(None),
        };

        if let QuestionType::Media(medias) = &question.question_type {
            if let Some(media) = medias.get(media_player.current_media) {
                return MessageResult(media.media_token.clone());
            }
        }

        MessageResult(None)
    }
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct NextMedia{
    pub websocket_session_id: WebsocketSessionId,
}



impl Handler<NextMedia> for Lobby {
    type Result = MessageResult<NextMedia>;
    
    fn handle(&mut self, msg: NextMedia, _: &mut Self::Context) -> Self::Result {
        let current = match self.jeopardy_board.current.clone() {
            Some(c) => c,
            None => return MessageResult(()),
        };

        let question = match self.jeopardy_board.get_question(current) {
            Some(q) => q,
            None => return MessageResult(()),
        };

        let mut state = match self.jeopardy_board.action_state.lock() {
            Ok(s) => s,
            Err(_) => return MessageResult(()), 
        };

        let media_state = match state.get_mut_media_player() {
            Some(mp) => mp,
            None => return MessageResult(()),
        };

        let medias = question.question_type.get_media();
        if medias.len() <= 0 {
            return MessageResult(());
        }
        if media_state.next(&medias, &msg.websocket_session_id){
            let status = media_state.status.clone();
            let event = WebsocketServerEvents::ActionState(ActionStateEvent::Media(ActionMediaEvent::ChangeState(status)));
            self.send_lobby_message(&event);
        } 
        MessageResult(())

    }

}


#[derive(Message)]
#[rtype(result = "()")]
pub struct BeforeMedia{
    pub websocket_session_id: WebsocketSessionId,
}



impl Handler<BeforeMedia> for Lobby {
    type Result = MessageResult<BeforeMedia>;
    
    fn handle(&mut self, msg: BeforeMedia, t: &mut Self::Context) -> Self::Result {
        let current = match self.jeopardy_board.current.clone() {
            Some(c) => c,
            None => return MessageResult(()),
        };

        let question = match self.jeopardy_board.get_question(current) {
            Some(q) => q,
            None => return MessageResult(()),
        };

        let mut state = match self.jeopardy_board.action_state.lock() {
            Ok(s) => s,
            Err(_) => return MessageResult(()), 
        };

        let media_state = match state.get_mut_media_player() {
            Some(mp) => mp,
            None => return MessageResult(()),
        };

        let medias = question.question_type.get_media();
        if medias.len() <= 0 {
            return MessageResult(());
        }
        if media_state.before(&msg.websocket_session_id){
            let status = media_state.status.clone();
            let event = WebsocketServerEvents::ActionState(ActionStateEvent::Media(ActionMediaEvent::ChangeState(status)));
            self.send_lobby_message(&event);
        } 
        MessageResult(())

    }

}







#[derive(Message)]
#[rtype(result = "()")]
pub struct BuzzerClicked{
    pub user_session_id: UserSessionId,
    pub current_time:  DateTime<Local>,
}
impl Handler<BuzzerClicked> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: BuzzerClicked,ctx: &mut Self::Context) -> Self::Result {
        let mut buzzer_state = self.jeopardy_board.buzzer_state.lock().expect("Failed to lock buzzer_state");

        if let BuzzerState::BuzzerOpen(ref mut state_map) = *buzzer_state {
            if state_map.contains_key(&msg.user_session_id) {
                println!("User {:?} has already buzzed", msg.user_session_id.id);
                return;
            }
            let grace_period = chrono::Duration::seconds(2);
            let grace_period2 = Duration::from_secs(2);

            if let Some(&first_buzz_time) = state_map.values().next() {
                if msg.current_time.signed_duration_since(first_buzz_time) > grace_period {
                    println!(
                        "User {:?} has buzzed too late by {:?}",
                        msg.user_session_id.id,
                        msg.current_time.signed_duration_since(first_buzz_time)
                    );
                    return;
                }
            }
            let clone_user_session_id = msg.user_session_id.clone();
            if state_map.len() == 0 {
                ctx.run_later(grace_period2, move |act, t| {
                    let buzzer_state = act.jeopardy_board.buzzer_state.lock().expect("Failed to lock buzzer_state");
                    if let BuzzerState::BuzzerOpen(_) = *buzzer_state {
                        println!("Buzzer grace period has ended BuzzeringStopped");
                        t.address().do_send(BuzzeringStopped{user_session_id:clone_user_session_id});
                    }
                });
            }



            state_map.insert(msg.user_session_id.clone(), msg.current_time);
            println!(
                "User {:?} buzzed in at time {:?}",
                msg.user_session_id.id, msg.current_time
            );
        }
    }
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct BuzzeringStarting {
    pub user_session_id: UserSessionId,
}
impl Handler<BuzzeringStarting> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: BuzzeringStarting,ctx: &mut Self::Context) -> Self::Result {
        let mut buzzer_state = self.jeopardy_board.buzzer_state.lock().expect("Failed to lock buzzer_state");

        match *buzzer_state {
            BuzzerState::BuzzerOpen(_) => {
                println!("Buzzer is already open");
            }
            BuzzerState::BuzzerdClosed(_) | BuzzerState::None => {
                *buzzer_state = BuzzerState::BuzzerOpen(HashMap::new());
                let event = WebsocketServerEvents::Board(BoardEvent::BuzzeringStarting);
                self.send_lobby_message(&event);
                println!("Buzzer has been opened");
            } 

        }

    }

}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BuzzeringStopped {
    pub user_session_id: UserSessionId
}
impl Handler<BuzzeringStopped> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: BuzzeringStopped, _: &mut Self::Context) -> Self::Result {
        let mut buzzer_state = self.jeopardy_board.buzzer_state.lock().expect("Failed to lock buzzer_state");
        if let BuzzerState::BuzzerOpen(ref mut state_map) = *buzzer_state {
            if state_map.len() == 0 {
                return;
            }
            let sorted_vec: Vec<UserSessionId> = state_map.iter().sorted_by(|a, b| a.1.cmp(b.1)).map(|(k, _)| k.clone()).collect();
            *buzzer_state = BuzzerState::BuzzerdClosed(sorted_vec.clone());
            let event = WebsocketServerEvents::Board(BoardEvent::BuzzeringClosed(sorted_vec));
            self.send_lobby_message(&event);
            println!("Buzzer has been closed");
        }
    }
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct BuzzerReset{
    pub user_session_id: UserSessionId
}
impl Handler<BuzzerReset> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: BuzzerReset, _: &mut Self::Context) -> Self::Result {
        let mut buzzer_state = self.jeopardy_board.buzzer_state.lock().expect("Failed to lock buzzer_state");
        *buzzer_state = BuzzerState::None;
        println!("Buzzer has been reset");
    }
}









pub async fn get_session(db: &Arc<MongoServer>, user_session_id: &UserSessionId) -> Option<UserSession> {
    db.find_user_session(&user_session_id).await
}

pub async fn get_sessions(db: &Arc<MongoServer>, user_session_ids: &Vec<UserSessionId>) -> Vec<UserSession>{
    let mut sessions = Vec::new();
    for user_session_id in user_session_ids {
        let session = get_session(&db, &user_session_id).await;
        if let Some(session) = session {
            sessions.push(session);
        }
    }
    sessions
}


pub async fn get_dto_sesion(db: &Arc<MongoServer>, user_session: &UserSession, data: &UserSessionData) -> DTOSession {
    let is_admin = match user_session.get_discord_id() {
        None => false,
        Some(discord_id) => db.is_admin(&discord_id).await,
    };
    user_session.clone().dto(&data.score, is_admin)
}


pub async fn get_dto_sessions(db: &Arc<MongoServer>, user_sessions:Vec<UserSession>, data: HashMap<UserSessionId, UserSessionData>) -> Vec<DTOSession> {
    let mut sessions = Vec::new();
    let admin = db.get_admins().await;
    for user_session in user_sessions {
        let is_admin = match user_session.get_discord_id() {
            None => false,
            Some(discord_id) => admin.iter().any(|admin| admin.discord_id.eq(&discord_id)),
        };
        let data = data.get(&user_session.user_session_id).expect("User session not found");
        let dto = user_session.dto(&data.score, is_admin);
        sessions.push(dto);
    }
    sessions
}
 

