
use serde::{Deserialize, Serialize};
use strum::Display;
use tsify_next::Tsify;
use std::hash::{Hash};
use std::string::ToString;

use crate::backend::ActionState;
use crate::dto::{DTOSession, DtoJeopardyBoard, DtoQuestion};

use super::ids::lobby::LobbyId;
use super::ids::usersession::UserSessionId;
use super::ids::websocketsession::WebsocketSessionId;
use super::Vector2D;


#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
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
pub enum ActionStateEvent {
    Media(ActionMediaEvent)


}
#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
pub enum ActionMediaEvent {
    Play,
    Pause,
    Resume,
}





#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
pub enum BoardEvent {
    CurrentBoard(DtoJeopardyBoard),
    CurrentQuestion(DtoQuestion, ActionState),
    UpdateCurrentQuestion(Option<Vector2D>),
    UpdateSessionScore(UserSessionId, i32),
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
pub enum WebsocketEvent {
    WebsocketJoined(WebsocketSessionId),
    WebsocketDisconnected(WebsocketSessionId),
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize, Display, Hash)]
pub enum SessionEvent {
    CurrentSessions(Vec<DTOSession>),
    SessionJoined(DTOSession),
    SessionsPing(Vec<WebsocketPing>),
    SessionDisconnected(UserSessionId),
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
pub enum WebsocketSessionEvent {
    Click(Vector2D),
    Back,
    AddUserSessionScore(UserSessionId, Vector2D),
    ViedeoEvent(VideoEvent),
    
}
#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
pub enum VideoEvent {
    Play,
    Pause(i64),
    Resume(i64),
}





#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Display)]
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