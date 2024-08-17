use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::wasm_lib::{hashs::validate::ValidateHash};

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








