use bytes::Bytes;
use chrono::{DateTime, Local};
use ids::discord::DiscordID;
use ids::usersession::UserSessionId;
use mongodb::bson::{doc, Bson, Document};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use twox_hash::XxHash;
use core::hash;
use std::f32::consts::E;
use std::hash::{Hash, Hasher};
use std::string::ToString;
use wasm_bindgen::prelude::*;

use crate::backend::{ActionState, MediaPlayer};
use crate::dto::FileChunk;

pub mod ids;
pub mod websocket_events;



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
#[derive(Tsify, Debug,Serialize,Deserialize ,Clone ,Hash,Eq, PartialEq, Default)]
pub struct FileData {
    chunks: Vec<FileChunk>,
    pub file_name: String,
    pub total_chunks: usize,
    pub file_type: String,
    pub chunk_hash: String,
    pub validate_hash: String,
    pub upload_data: DateTime<Local>,
    pub uploader: UserSessionId,
}






    


impl FileData {

    pub fn new(chunks: Vec<FileChunk>, file_name: String, total_chunks: usize, file_type: String, chunk_hash: String, validate_hash: String, upload_data: DateTime<Local>, uploader: UserSessionId) -> Self {
        FileData {
            chunks,
            file_name,
            total_chunks,
            file_type,
            chunk_hash,
            validate_hash,
            upload_data,
            uploader,
        }
    }

    pub fn current_chunks(&self) -> usize {
        self.chunks.len()
    }
    
    pub fn is_valid(&self) -> bool {
        self.chunks.len() == self.total_chunks && self.validate_hash == self.video_chunk_hash()
    }

    pub fn get_chunk(&self, index: usize) -> Option<&FileChunk> {
        self.chunks.get(index)
    }

   
    pub fn try_to_add_chunk(&mut self, chunk: FileChunk) -> bool {
        let index_chunk = self.get_chunk(chunk.index);
        match index_chunk {
            Some(index) => {
                if index.chunk == chunk.chunk || index.hash == chunk.hash {
                    return false
                } 
                self.add_chunk(chunk)
            }
            None => self.add_chunk(chunk)
            
        }
    }

    fn add_chunk(&mut self, chunk: FileChunk) -> bool {
        if chunk.index >= self.total_chunks || !chunk.is_valid() {
            return false
        }
        self.chunks.push(chunk);
        self.chunk_hash = self.update_video_hash();
        true
    }
    


    pub fn video_chunk_hash(&self) -> String {
        let mut hasher = XxHash::with_seed(0); // Seed is optional
        hasher.write(&self.chunks.iter().map(|x| x.chunk.clone()).flatten().collect::<Vec<u8>>());
        hasher.finish().to_string()
    }

    pub fn update_video_hash(&mut self) -> String {
        self.chunk_hash = self.video_chunk_hash();
        self.chunk_hash.clone()
    }




}

