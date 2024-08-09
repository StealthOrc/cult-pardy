use api::DTOFileToken;
use bytes::Bytes;
use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize};
use tsify_next::Tsify;
use twox_hash::XxHash;
use std::collections::HashSet;
use std::fmt::Binary;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::string::ToString;
use wasm_bindgen::prelude::*;

use crate::wasm_lib::hashs::filechunk::FileChunkHash;
use crate::wasm_lib::hashs::validate::ValidateHash;
use crate::wasm_lib::ids::discord::DiscordID;
use crate::wasm_lib::ids::usersession::{self, UserSessionId};
use crate::wasm_lib::token::file::FileToken;
use crate::wasm_lib::{DiscordUser, FileData, QuestionType, Vector2D};


pub mod board;
pub mod api;
pub mod file;



#[derive(Tsify, Debug, Clone,Serialize,Deserialize, Hash,Eq, PartialEq, Default)]
pub struct DTOFileData {
    pub file_name: String,
    pub file_type: String,
    pub total_chunks: usize,
    pub file_chunks_hashs: Vec<FileChunkHash>,
    pub validate_hash: ValidateHash,
}


impl DTOFileData {
    
    pub fn to_file_data(self, discord_id:&DiscordID) -> FileData {
        FileData::new(self.file_chunks_hashs, self.file_name, self.total_chunks, self.file_type, self.validate_hash, discord_id)
    }

}


#[derive(Tsify, Debug, Clone,Serialize,Deserialize, Hash,Eq, PartialEq, Default)]
pub struct DTOFileChunk {
    pub file_name: String,
    pub index: usize,
    pub chunk: Bytes,
    pub validate_hash: ValidateHash,
}

impl DTOFileChunk {


    pub fn to_chunk_hash(&self) -> FileChunkHash {
        let mut hasher = XxHash::with_seed(0); // Seed is optional
        hasher.write(&self.chunk);
        FileChunkHash {
            hash: hasher.finish().to_string(),
        }
    }
    
}

