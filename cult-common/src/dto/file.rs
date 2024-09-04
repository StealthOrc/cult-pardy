
use std::fs::File;

use bytes::Bytes;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use utoipa::ToSchema;
use wasm_bindgen::prelude::*;

use crate::wasm_lib::hashs::validate::ValidateHash;
use crate::wasm_lib::ids::discord::DiscordID;
use crate::wasm_lib::NumberScope;


#[derive(Tsify,Debug, Serialize, Deserialize, Default, ToSchema)]
pub struct FileMultiPart {
    #[tsify(optional)]
    pub file_name: Option<String>,
    #[tsify(optional)]
    pub file_type: Option<String>,
    #[tsify(optional)]
    pub validate_hash: Option<ValidateHash>,
    #[tsify(optional)]
    pub data: Option<Vec<u8>>,
    #[tsify(optional)]
    pub uploader_id: Option<DiscordID>,
}

impl FileMultiPart {
   pub fn is_valid(&self) -> bool {
       self.file_name.is_some() && self.file_type.is_some() && self.validate_hash.is_some() && self.uploader_id.is_some()
   }
}


#[derive(Tsify,Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DTOCFile{
    pub file_name: String,
    pub file_type: String,
    pub data: Bytes,
}




#[derive(Debug, Clone,Serialize,Deserialize, Hash,Eq, PartialEq, ToSchema)]
pub struct DTOFileData {
    pub length : usize,
    pub upload_date: String,
    pub file_name: String,
    pub metadata: DTOFileMetadata,
}




#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct DTOFileMetadata {
    pub file_type: String,
    pub validate_hash: ValidateHash,
    pub uploader: DiscordID,
}