
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::wasm_lib::hashs::validate::ValidateHash;
use crate::wasm_lib::ids::discord::DiscordID;


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
