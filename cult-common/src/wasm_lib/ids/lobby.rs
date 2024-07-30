use std::fmt::{Display, Formatter};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Tsify, Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct LobbyId {
    #[wasm_bindgen(skip)]
    pub id: String,
}

#[wasm_bindgen]
impl LobbyId {

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(constructor)]
    pub fn of(id: String) -> Self {
        LobbyId { id }
    }

    pub fn from_str(id: &str) -> Self {
        LobbyId { id: id.to_string() }
    }

    pub fn random() -> Self {
        let id = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        LobbyId { id }
    }
}

impl Display for LobbyId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

