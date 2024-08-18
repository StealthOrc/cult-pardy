use rand::random;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use utoipa::ToSchema;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Tsify,Default, Debug,Clone, PartialEq, Eq, Hash, Serialize, Deserialize,ToSchema)]
pub struct UserSessionId {
     pub id: String,
}
impl UserSessionId {

    pub fn id(self) -> usize {
        return self.id.parse::<usize>().unwrap();
    }

    pub fn of(id: usize) -> Self {
        UserSessionId { id: id.to_string() }
    }

    pub fn from_string(id: String) -> Self {
        UserSessionId { id }
    }
    
    pub fn from_str(id: &str) -> Self {
        UserSessionId { id: id.to_string() }
    }

    pub fn server() -> Self {
        UserSessionId {
            id: "000000000000000".to_string(),
        }
    }
    pub fn random() -> Self {
        UserSessionId {
            id: random::<usize>().to_string(),
        }
    }
}
