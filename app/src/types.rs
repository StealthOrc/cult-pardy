use cult_common::{DiscordUser, WebsocketSessionEvent};
use yew::Callback;

// Message for Yew App
#[derive(Clone)]
pub enum AppMsg {
    SendWebsocketMessage(WebsocketSessionEvent),
    BoardUnloaded,
    BoardLoaded,
    ShowQuestion,
    UpdateUserInfo(Vec<DiscordUser>),
}

pub type WebsocketCallback = Callback<WebsocketSessionEvent>;
pub type DiscordUserList = Option<Vec<DiscordUser>>;
