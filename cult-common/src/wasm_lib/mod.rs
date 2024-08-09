use bytes::Bytes;
use chrono::{DateTime, Local};
use hashs::filechunk::FileChunkHash;
use hashs::filedata::FileDataHash;
use hashs::validate::ValidateHash;
use ids::discord::DiscordID;
use ids::usersession::{self, UserSessionId};
use serde::{Deserialize, Serialize};
use token::file::FileToken;
use tsify_next::Tsify;
use twox_hash::XxHash;
use core::hash;
use std::f32::consts::E;
use std::hash::{Hash, Hasher};
use std::string::ToString;
use wasm_bindgen::prelude::*;

use crate::backend::{ActionState, MediaPlayer};
use crate::dto::api::DTOFileToken;

pub mod ids;
pub mod websocket_events;
pub mod hashs;
pub mod token;


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


impl QuestionType {
    pub fn get_action_state(self: &QuestionType) -> ActionState {
        match self {
            QuestionType::Media(_) =>  ActionState::MediaPlayer(MediaPlayer::default()),
            _ => ActionState::None,
        }
    }
    
}






#[derive(Tsify, Debug,Serialize,Deserialize ,Clone ,Hash,Eq, PartialEq, Default)]
pub struct FileData {
    file_chunks_hashs: Vec<FileChunkHash>,
    pub file_name: String,
    pub total_chunks: usize,
    pub file_type: String,
    pub filedata_hash: FileDataHash,
    pub validate_hash: ValidateHash,
    pub upload_data: DateTime<Local>,
    pub uploader: DiscordID,
    pub file_token: FileToken,
}



impl FileData {

    pub fn new(file_chunks_hashs: Vec<FileChunkHash>, file_name: String, total_chunks: usize, file_type: String, validate_hash:ValidateHash,uploader: &DiscordID) -> Self {
        let filedata_hash = FileDataHash::default();
        let upload_data = Local::now();
        FileData {
            file_chunks_hashs,
            file_name,
            total_chunks,
            file_type,
            filedata_hash,
            validate_hash,
            upload_data,
            uploader: uploader.clone(),
            file_token: FileToken::new(),
        }
    }

    pub fn containts_file_chunk_hash(&self, hash: &ValidateHash) -> bool {
        self.file_chunks_hashs.iter().any(|x| x.hash == hash.get_hash())
    }

    pub fn get_hashs(&self) -> Vec<FileChunkHash> {
        self.file_chunks_hashs.clone()
    }

    pub fn validate_file_token(&self, token: &DTOFileToken) -> bool {
        self.file_token.token.eq(&token.token) && !self.file_token.is_expired()
    }




}

