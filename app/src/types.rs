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
use crate::app;
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





pub fn process_cookies(res: &Response, old_session_token: String, old_user_session_id: &UserSessionId) {
    let cookies = res.headers().get("Cookie");
    if let Some(cookies) = cookies{
        let cookies = cookies.to_string();
        let cookies = cookies.split(", ").collect::<Vec<&str>>();
        let new_user_session_id = cookies.iter().find(|&&cookie| cookie.starts_with("user-session-id=")).unwrap().trim_start_matches("user-session-id=").to_string();
        let new_session_token = cookies.iter().find(|&&cookie| cookie.starts_with("session-token=")).unwrap().trim_start_matches("session-token=").to_string();
        update_cookie("session-token", &old_session_token, &new_session_token);
        update_cookie("user-session-id", &old_user_session_id.id, &new_user_session_id);
    }
}

fn update_cookie(name:&str,old_value:&String, new_value:&String){
    if !old_value.eq(&new_value.as_str()) && (new_value.len().eq(&19) | new_value.len().eq(&30))  {
        log!(format!("UDPATE! {:?}", name));
        set(&name,&new_value, &CookieOptions {
            path: Some("/"),
            domain: None,
            expires: None,
            secure: true,
            same_site: Default::default(),
        });
    }

}