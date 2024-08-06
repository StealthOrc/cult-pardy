use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use super::{filechunk::FileChunkHash, filedata::FileDataHash};




#[derive(Tsify,Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct ValidateHash {
    hash: String,
}

impl ValidateHash {

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn validate_file_chunk(&self, hash: &FileChunkHash) -> bool {
        self.hash == hash.get_hash()
    }

    pub fn validate_file_data(&self, hash: &FileDataHash) -> bool {
        self.hash == hash.get_hash()
    }

}