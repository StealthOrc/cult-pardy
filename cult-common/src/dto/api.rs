use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use utoipa::ToSchema;







#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default, ToSchema)]
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








