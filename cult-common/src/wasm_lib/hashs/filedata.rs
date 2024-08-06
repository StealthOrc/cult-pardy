use std::default;

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;




#[derive(Tsify,Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct FileDataHash {
    hash: String,
}

impl FileDataHash {

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

}