use ids::{discord::DiscordID, usersession::UserSessionId, websocketsession::{self, WebsocketSessionId}};
use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use tsify_next::Tsify;
use utoipa::ToSchema;
use websocket_events::MediaState;
use std::{cmp::min, collections::HashMap, hash::Hash};
use std::string::ToString;
use wasm_bindgen::prelude::*;

use crate::backend::{ActionState, ActionStateType};

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
    Media(Media),
    Youtube(String),
    #[default]
    Question
}



impl QuestionType {
    pub fn get_action_state(self: &QuestionType,websocketsession:&WebsocketSessionId) -> ActionState {
        match self {
            QuestionType::Media(_) =>  ActionState {
                state: ActionStateType::MediaPlayer(MediaState::new(websocketsession)),
                current_type: Some(self.clone()),
            },
            QuestionType::Youtube(_) => ActionState {
                state: ActionStateType::MediaPlayer(MediaState::new(websocketsession)),
                current_type: Some(self.clone()),
            },
            _ => ActionState {
                state: ActionStateType::None,
                current_type: None,
            }
        }
    }
}
#[derive(Tsify,Debug, Clone, Serialize, Eq, PartialEq, Default,ToSchema)]
pub struct Media {
    pub media_type: MediaType,
    pub name: String,
    pub media_token: MediaToken,
    pub media_loaded: Vec<UserSessionId>
}

impl Media {

    pub fn new(media_type: MediaType, name: String) -> Self {
        Media {
            media_type,
            name,
            media_token: MediaToken::random(),
            media_loaded: Vec::new(),
        }
    }
    
}

impl<'de> Deserialize<'de> for Media {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,{

        #[derive(Deserialize)]
        struct MediaVisitor{
            pub media_type: MediaType,
            pub name: String,
        }
        let media = MediaVisitor::deserialize(deserializer)?;
        Ok(Media {
            media_type: media.media_type,
            name: media.name,
            media_token: MediaToken::random(),
            media_loaded: Vec::new(),
        })
    }
}





#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default,ToSchema)]
pub struct MediaToken {
    pub token: String,
}

impl MediaToken {
    pub fn new(token: String) -> Self {
        MediaToken { token }
    }

    pub fn random() -> Self {
        MediaToken { token: random::<usize>().to_string() }
    }
}






#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default,ToSchema)]
#[tsify(namespace)]
pub enum MediaType {
    Image,
    Video(Vec<VideoType>),
    Audio,
    Text,
    Pdf,
    #[default]
    Unknown,
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default,ToSchema)]
#[tsify(namespace)]
pub enum VideoType {
    #[default]
    None,
    TimeSlots(Vec<NumberScope>),
    Mute,
    Slowmotion(usize),
    
}

impl VideoType {
    
    pub fn time_slots(vec: Vec<NumberScope>) -> VideoType {
       if vec.len() == 0 {
           VideoType::None
       } else {
           VideoType::TimeSlots(vec)
       }
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