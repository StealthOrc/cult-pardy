
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

use crate::wasm_lib::hashs::validate::ValidateHash;
use crate::wasm_lib::ids::discord::DiscordID;
use crate::wasm_lib::ids::usersession::{self, UserSessionId};
use crate::wasm_lib::{DiscordUser, QuestionType, Vector2D};


#[derive(Tsify,Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FileMultiPart {
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub validate_hash: Option<ValidateHash>,
    pub uploader_id: Option<DiscordID>,
}

impl FileMultiPart {
   pub fn is_valid(&self) -> bool {
       self.file_name.is_some() && self.file_type.is_some() && self.validate_hash.is_some() && self.uploader_id.is_some()
   }
}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FileChunk {
    pub files_id: String,
    pub file_type: String,
    pub n: usize,
    pub data: Bytes,
}


#[derive(Tsify,Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DTOCFile{
    pub file_name: String,
    pub file_type: String,
    pub data: Bytes,
}
