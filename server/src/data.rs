use cult_common::wasm_lib::ids::usersession::UserSessionId;
use serde::{Deserialize, Serialize};
use crate::servers::game::SessionToken;





#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct SessionRequest {
    pub user_session_id: UserSessionId,
    pub session_token: SessionToken,
}
