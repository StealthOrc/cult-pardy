use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::wasm_lib::token::File::FileToken;

use super::file;





#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct ApiResponse {
    pub success: bool,

}
impl ApiResponse {
    pub fn new(success: bool) -> Self {
        ApiResponse { success }
    }
    pub fn of(success: bool) -> Self {
        ApiResponse { success }
    }
}








#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, Hash,PartialEq, Default)]
pub struct DTOFileToken {
    pub token: String,
}


#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum FileDataReponse {
    Successful(DTOFileToken),
    Failed(String),
}

impl FileDataReponse {
    pub fn successful(file_token: &FileToken) -> Self {
        FileDataReponse::Successful(file_token.to_dto_file_token())
    }
    pub fn failed(error: String) -> Self {
        FileDataReponse::Failed(error)
    }

}