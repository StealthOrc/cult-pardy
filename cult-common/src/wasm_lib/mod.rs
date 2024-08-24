use ids::{discord::DiscordID, usersession::UserSessionId, websocketsession::{self, WebsocketSessionId}};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use utoipa::ToSchema;
use websocket_events::MediaState;
use std::{cmp::min, hash::Hash};
use std::string::ToString;
use wasm_bindgen::prelude::*;

use crate::backend::{ActionState};

pub mod ids;
pub mod websocket_events;
pub mod hashs;


#[derive(Tsify,Clone, Copy,Serialize,Deserialize)]
#[tsify(namespace)] 
pub enum JeopardyMode {
    //3x3
    SHORT,
    //5x5
    NORMAL,
    //7x7
    LONG,
}

#[derive(Tsify,Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Vector2D {
    pub x: usize,
    pub y: usize,
}



impl JeopardyMode {
    pub fn field_size(self: JeopardyMode) -> usize {
        match self {
            JeopardyMode::SHORT => 3,
            JeopardyMode::NORMAL => 5,
            JeopardyMode::LONG => 7,
        }
    }
}


#[derive(Tsify, Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, ToSchema)]
pub struct DiscordUser {
    // Fields are private and not exposed to JavaScript
    pub discord_id: DiscordID,
    pub username: String,
    pub avatar_id: String,
    pub discriminator: String,
    pub global_name: String,
}

impl DiscordUser {
    pub fn discord_id(&self) -> DiscordID {
        self.discord_id.clone()
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn avatar_id(&self) -> String {
        self.avatar_id.clone()
    }

    pub fn discriminator(&self) -> String {
        self.discriminator.clone()
    }

    pub fn global_name(&self) -> String {
        self.global_name.clone()
    }

    pub fn avatar_image_url(&self) -> String {
        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.jpg",
            self.discord_id.id(), self.avatar_id
        )
    }
}






#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default,ToSchema)]
#[tsify(namespace)]
pub enum QuestionType {
    Video(Blob),
    Youtube(String),
    #[default]
    Question
}



impl QuestionType {
    pub fn get_action_state(self: &QuestionType, user_session_id:&UserSessionId, websocketsession:&WebsocketSessionId) -> ActionState {
        match self {
            QuestionType::Video(_) =>  ActionState::MediaPlayer(MediaState::new(websocketsession)),
            _ => ActionState::None,
        }
    }
    
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default,ToSchema)]
pub struct Blob {
    pub name: String,
    pub range: Vec<NumberScope>,
}

impl Blob {

    pub fn new(name: String, range: Vec<NumberScope>) -> Self {
        Blob { name, range }
    }

    pub fn new_empty(name: String) -> Self {
        Blob { name, range: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        self.range.iter().all(|range| !range.is_empty())
    }

    pub fn is_valid_range(&self, range: &NumberScope) -> bool {
        self.range.iter().any(|r| r.overlaps(range))
    }

    pub fn intersection(&self, range: &NumberScope) -> Option<NumberScope> {
        self.range.iter().filter_map(|r| r.intersection(range)).next()
    }



    
}



#[derive(Tsify, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct NumberScope {
    pub start: usize,
    pub end: usize,
}

impl NumberScope {
    pub fn new(start: usize, end: usize) -> Self {
        NumberScope { start, end }
    }

    pub fn overlaps(&self, other: &NumberScope) -> bool {
        self.start <= other.end && self.end >= other.start
    }

    pub fn intersection(&self, other: &NumberScope) -> Option<NumberScope> {
        if self.overlaps(other) {
            Some(NumberScope::new(
                usize::max(self.start, other.start),
                usize::min(self.end, other.end),
            ))
        } else {
            None
        }
    }

    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}