use std::fmt::{Display, Formatter};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use utoipa::ToSchema;
use wasm_bindgen::prelude::wasm_bindgen;


#[derive(Tsify, Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct LobbyId {
    pub id: String,
}

impl LobbyId {


    pub fn id(&self) -> String {
        self.id.clone()
    }

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


