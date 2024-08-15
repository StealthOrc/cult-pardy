
use bytes::Bytes;
use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize};
use tsify_next::Tsify;
use twox_hash::XxHash;
use std::collections::HashSet;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::string::ToString;
use wasm_bindgen::prelude::*;

use crate::wasm_lib::hashs::filechunk::FileChunkHash;
use crate::wasm_lib::hashs::validate::ValidateHash;
use crate::wasm_lib::ids::discord::DiscordID;
use crate::wasm_lib::ids::usersession::{self, UserSessionId};
use crate::wasm_lib::{DiscordUser, FileData, QuestionType, Vector2D};

use super::DTOFileChunk;





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

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DTOCFile{
    pub file_name: String,
    pub file_type: String,
    pub chunks: Vec<DTOFileChunk>,
    pub validate_hash: ValidateHash,
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FileMultiPart {
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub validate_hash: Option<ValidateHash>,
    pub creator_id: Option<DiscordID>,
}

impl FileMultiPart {
   

   pub fn is_valid(&self) -> bool {
       self.file_name.is_some() && self.file_type.is_some() && self.validate_hash.is_some() && self.creator_id.is_some()
   }
    
}