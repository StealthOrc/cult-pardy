use rand::random;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Tsify,Default, Debug,Clone, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub struct UserSessionId {
    #[wasm_bindgen(skip)]
     pub id: String,
}
#[wasm_bindgen]
impl UserSessionId {

    #[wasm_bindgen(getter)]
    pub fn id(self) -> usize {
        return self.id.parse::<usize>().unwrap();
    }

    #[wasm_bindgen(constructor)]
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

impl Serialize for UserSessionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id)
    }
}

impl<'de> Deserialize<'de> for UserSessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        Ok(UserSessionId { id })
    }
}