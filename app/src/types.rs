use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use cult_common::{DiscordUser, UserSessionId, WebsocketServerEvents, WebsocketSessionEvent};
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

pub type UserList = HashMap<UserSessionId, Option<DiscordUser>>;
