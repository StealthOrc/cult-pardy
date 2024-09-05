use ids::{discord::DiscordID, usersession::UserSessionId, websocketsession::{self, WebsocketSessionId}};
use rand::random;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use utoipa::ToSchema;
use websocket_events::MediaStatus;
use std::{cmp::min, hash::Hash};
use std::string::ToString;
use wasm_bindgen::prelude::*;

use crate::backend::{ActionState, MediaState};

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
    Media(Vec<Media>),
    Youtube(String),
    #[default]
    Question
}





impl QuestionType {
    pub fn get_default_actionstate(self: &QuestionType,websocket_session_id:&WebsocketSessionId) -> ActionState {
        match self {
            QuestionType::Media(media) => {
                let media = media.get(0).expect("Media is empty");
                match media.media_type {
                    MediaType::Video(_) => {
                        ActionState::MediaPlayer(MediaState::new(websocket_session_id))
                    }
                    _ => ActionState::None,
                }
            }
            _ => ActionState::None,
        }
    }

    pub fn get_media(self: &QuestionType) -> Vec<Media> {
        match self {
            QuestionType::Media(media) => media.clone(),
            _ => vec![],
        }
    }


}

#[derive(Tsify,Debug, Clone, Serialize, Eq, PartialEq, Default,ToSchema)]
pub struct Media {
    pub media_type: MediaType,
    pub name: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub media_token: Option<MediaToken>,
}

impl Media {

    pub fn new(media_type: MediaType, name: String) -> Self {
        Media {
            media_type,
            name,
            media_token: Some(MediaToken::random()),
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
            media_token: Some(MediaToken::random()),
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


impl MediaType {

    pub fn from_string(string_type: &str) -> MediaType {

        let type_vec: Vec<&str> = string_type.split("/").collect();
        let media_type = match type_vec[..] {
            [media_type, _] => media_type.to_lowercase(),
            _ => "unknown".to_owned(),
        };
        match media_type.as_str() {
            "image" => MediaType::Image,
            "audio" => MediaType::Audio,
            "text" => MediaType::Text,
            "pdf" => MediaType::Pdf,
            "video" => MediaType::Video(vec![]),
            _ => MediaType::Unknown,
        }
    }
    
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










#[derive(Tsify, Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
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