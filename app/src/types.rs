use cult_common::{
    DTOSession, DiscordUser, UserSessionId, WebsocketServerEvents, WebsocketSessionEvent,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
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
