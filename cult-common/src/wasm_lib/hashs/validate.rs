use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use utoipa::ToSchema;

use super::{filedata::FileDataHash};




#[derive(Tsify,Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default, ToSchema)]
pub struct ValidateHash {
    hash: String,
}

impl ValidateHash {

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn validate_file_chunk(&self, hash: &FileDataHash) -> bool {
        self.hash == hash.get_hash()
    }

    pub fn new(hash: String) -> Self {
        ValidateHash { hash }
    }

}