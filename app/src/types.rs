use cult_common::{DTOSession, DiscordUser, UserSessionId, WebsocketServerEvents, WebsocketSessionEvent, PROTOCOL, LOCATION};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use gloo_console::log;
use gloo_net::http::{Headers, Response};
use ritelinked::LinkedHashMap;
use wasm_cookies::CookieOptions;
use wasm_cookies::cookies::{get, set};
use yew::Callback;
use crate::profilepanel::_PlayerPanelProperties::user_session_id;

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




