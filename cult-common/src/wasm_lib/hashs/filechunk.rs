use serde::{Deserialize, Serialize};
use tsify_next::Tsify;




#[derive(Tsify,Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct FileChunkHash {
    pub hash: String,
}

impl FileChunkHash {

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

}