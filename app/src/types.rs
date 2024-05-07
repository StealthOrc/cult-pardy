use cult_common::{Vector2D, WebsocketEvent, WebsocketSessionEvent};
use yew::Callback;

// Message for Yew App
#[derive(Clone, Copy)]
pub enum AppMsg {
    SendWebsocketMessage(WebsocketSessionEvent),
    BoardUnloaded,
    BoardLoaded,
    ShowQuestion,
}

pub type WebsocketCallback = Callback<WebsocketSessionEvent>;
