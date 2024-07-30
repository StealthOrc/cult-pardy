use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Tsify, Default, Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct DiscordID {
    pub id: String,
}

impl DiscordID {

    pub fn id(&self) -> String {
        self.id.clone()
    }
    
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }
    pub fn new(id: String) -> Self {
        DiscordID { id }
    }

    pub fn of_str(id: &str) -> Self {
        DiscordID { id: id.to_string() }
    }

    pub fn server() -> Self {
        DiscordID {
            id: "000000000000000".to_string(),
        }
    }
}

