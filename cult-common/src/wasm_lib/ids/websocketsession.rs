use rand::random;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;


#[derive(Tsify,Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct WebsocketSessionId {
    pub id: String,
}
impl WebsocketSessionId {
    pub fn id(self) -> usize {
        return self.id.parse::<usize>().unwrap();
    }

    pub fn random() -> Self {
        WebsocketSessionId {
            id: random::<usize>().to_string(),
        }
    }
    pub fn of(id: usize) -> Self {
        WebsocketSessionId { id: id.to_string() }
    }
    pub fn from_string(id: String) -> Self {
        WebsocketSessionId { id: id.to_string() }
    }
    pub fn from_str(id: &str) -> Self {
        let id = id.parse::<usize>().expect("Can´t convert String to usize");
        WebsocketSessionId { id: id.to_string() }
    }
}



