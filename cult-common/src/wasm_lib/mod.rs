use ids::discord::DiscordID;
use ids::usersession::UserSessionId;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use std::hash::{Hash, Hasher};
use std::string::ToString;
use wasm_bindgen::prelude::*;

pub mod ids;
pub mod websocketevents;



#[derive(Tsify,Clone, Copy,Serialize,Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum JeopardyMode {
    //3x3
    SHORT,
    //5x5
    NORMAL,
    //7x7
    LONG,
}

#[derive(Tsify,Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Vector2D {
    pub x: usize,
    pub y: usize,
}



#[wasm_bindgen]
impl JeopardyMode {
    #[wasm_bindgen]
    pub fn field_size(self: JeopardyMode) -> usize {
        match self {
            JeopardyMode::SHORT => 3,
            JeopardyMode::NORMAL => 5,
            JeopardyMode::LONG => 7,
        }
    }
}


#[derive(Tsify, Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
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






#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum QuestionType {
    Media(String),
    #[default]
    Question,
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct ApiResponse {
    pub success: bool,

}
impl ApiResponse {
    pub fn new(success: bool) -> Self {
        ApiResponse { success }
    }
    pub fn of(success: bool) -> Self {
        ApiResponse { success }
    }
}
