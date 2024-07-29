use ids::discord::DiscordID;
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


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[wasm_bindgen]
pub struct DiscordUser {
    // Fields are private and not exposed to JavaScript
    #[wasm_bindgen(skip)]
    pub discord_id: DiscordID,
    #[wasm_bindgen(skip)]
    pub username: String,
    #[wasm_bindgen(skip)]
    pub avatar_id: String,
    #[wasm_bindgen(skip)]
    pub discriminator: String,
    #[wasm_bindgen(skip)]
    pub global_name: String,
}

#[wasm_bindgen]
impl DiscordUser {
    #[wasm_bindgen(getter)]
    pub fn discord_id(&self) -> DiscordID {
        self.discord_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn username(&self) -> String {
        self.username.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn avatar_id(&self) -> String {
        self.avatar_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn discriminator(&self) -> String {
        self.discriminator.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn global_name(&self) -> String {
        self.global_name.clone()
    }

    #[wasm_bindgen]
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

#[derive(Tsify, Serialize, Deserialize)]
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