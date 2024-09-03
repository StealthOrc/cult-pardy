
use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use strum::Display;
use tsify_next::Tsify;
use std::hash::{Hash};
use std::string::ToString;

use crate::backend::ActionState;
use crate::dto::board::{DTOSession, DtoJeopardyBoard, DtoQuestion};

use super::ids::lobby::LobbyId;
use super::ids::usersession::UserSessionId;
use super::ids::websocketsession::{self, WebsocketSessionId};
use super::Vector2D;


#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum WebsocketServerEvents {
    Board(BoardEvent),
    Websocket(WebsocketEvent),
    Session(SessionEvent),
    Error(WebsocketError),
    ActionState(ActionStateEvent),
    Text(String),
}

impl WebsocketServerEvents {
    pub fn event_name(self) -> String {
        let wse = self.to_string();
        let event = match self {
            WebsocketServerEvents::Board(event) => event.to_string(),
            WebsocketServerEvents::Websocket(event) => event.to_string(),
            WebsocketServerEvents::Session(event) => event.to_string(),
            WebsocketServerEvents::Error(event) => event.to_string(),
            WebsocketServerEvents::Text(event) => event.to_string(),
            WebsocketServerEvents::ActionState(event) => event.to_string().to_string(),
        };

        format!("{} -> {} ", wse, event)
    }
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum ActionStateEvent {
    Media(ActionMediaEvent),
    SyncForward(f64),
    SyncBackward(i64),

}
#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum ActionMediaEvent {
    Play,
    Pause,
    Resume,
    ChangeState(MediaStatus),
}

#[derive(Tsify,Debug, Clone, Serialize,Deserialize,)]
pub struct MediaStatus {
    pub video_timestamp: f64,
    pub last_updated: f64,
    pub playing: bool,
    pub global_timestamp: f64,
    pub interaction_id: WebsocketSessionId,
}



impl MediaStatus {
    pub fn new(websocketsession:&WebsocketSessionId) -> Self {
        MediaStatus {
            video_timestamp: 0.0,
            last_updated: Local::now().timestamp_millis() as f64,
 
            playing: false,
            global_timestamp: 0.0,
            interaction_id: websocketsession.clone(),
        }
    }




    
}




#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum BoardEvent {
    CurrentBoard(DtoJeopardyBoard),
    CurrentQuestion(DtoQuestion, ActionState),
    UpdateCurrentQuestion(Option<Vector2D>),
    UpdateSessionScore(UserSessionId, i32),
    BuzzeringStarting,
    BuzzeringClosed(Vec<UserSessionId>),
    BuzzeringReset,
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum WebsocketEvent {
    WebsocketJoined(WebsocketSessionId),
    WebsocketID(WebsocketSessionId),
    WebsocketDisconnected(WebsocketSessionId),
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize, Display, Hash)]
#[tsify(namespace)] 
pub enum SessionEvent {
    CurrentSessions(Vec<DTOSession>),
    SessionJoined(DTOSession),
    SessionsPing(Vec<WebsocketPing>),
    SessionPing(WebsocketPing),
    SessionDisconnected(UserSessionId),
}













#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum WebsocketSessionEvent {
    ChooseQuestion(Vector2D),
    Back,
    AddUserSessionScore(UserSessionId, Vector2D),
    MediaEvent(MediaEvent),
    BuzzoringEvent(BuzzorEvent),
}




#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)]
pub enum BuzzorEvent {
    BuzzorClick,
    BuzzorStarting,
    BuzzorStop,
    BuzzorReset,
}







#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum MediaEvent {
    VideoEvent(VideoEvent),
    SyncBackwardRequest,
    SyncForwardRequest(f64)
}





#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum VideoEvent {
    Play,
    Pause(f64),
    Resume,
    ChangeState(MediaStatus),
}





#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
#[tsify(namespace)] 
pub enum WebsocketError {
    LobbyNotFound(LobbyId),
    SessionNotFound(UserSessionId),
    GameStarted(LobbyId),
    NotAuthorized,
    WebsocketCrashed,
    UNKNOWN(String),
}


#[derive(Tsify, Serialize, Clone,Deserialize, Debug, Hash)]
pub struct WebsocketPing {
    pub user_session_id: UserSessionId,
    pub ping : i64,
}