use cult_common::dto::DTOSession;
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use cult_common::wasm_lib::websocketevents::{WebsocketServerEvents, WebsocketSessionEvent};
use ritelinked::LinkedHashMap;
use yew::Callback;

// Message for Yew App
#[derive(Clone)]
pub enum AppMsg {
    SendWebsocketMessage(WebsocketSessionEvent),
    HandleWebsocketEvent(WebsocketServerEvents),
    BoardUnloaded,
    BoardLoaded,
}

pub type WebsocketCallback = Callback<WebsocketSessionEvent>;
pub type OptionalWebsocketCallback = Option<WebsocketCallback>;

pub type UserList = LinkedHashMap<UserSessionId, DTOSession>;




