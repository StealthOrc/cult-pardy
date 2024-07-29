use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Tsify, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub struct DiscordID {
    #[wasm_bindgen(skip)]
    pub id: String,
}

#[wasm_bindgen]
impl DiscordID {

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    
    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }
    #[wasm_bindgen(constructor)]
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

impl Serialize for DiscordID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id)
    }
}

impl<'de> Deserialize<'de> for DiscordID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        Ok(DiscordID { id })
    }
}