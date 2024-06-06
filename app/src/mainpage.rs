use futures::TryFutureExt;
use gloo_console::log;
use gloo_net::http::{Headers, Request};
use std::path::Path;
use wasm_cookies::cookies::{get, set};
use wasm_cookies::CookieOptions;
use web_sys::window;
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::game::app;
use cult_common::{
    get_false, DiscordUser, LobbyId, UserSessionId, WebsocketServerEvents, LOCATION, PROTOCOL,
};
use yew_router::prelude::RouterScopeExt;

#[derive(Properties, PartialEq)]
pub struct MainPage {
    is_logged_in: bool,
    is_admin: bool,
    join_code: LobbyId,
}

pub enum Msg {
    Loaded(bool),
    Login,
}

impl Component for MainPage {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        _ctx.link().send_future(async {
            let user_session_id = UserSessionId::from_string(
                get(&app::cookie_string(), "user-session-id")
                    .expect("could not get cookie")
                    .expect("could not get cookie from user"),
            );
            let session_token = get(&app::cookie_string(), "session-token")
                .expect("could not get cookie")
                .expect("could not get cookie from user");
            let _request_url = format!(
                "{}/api/discord_session?user-session-id={}&session-token={}",
                format!("{}{}", PROTOCOL, LOCATION),
                user_session_id.id,
                session_token
            );
            let resp = Request::get(&_request_url).send().await;
            match resp {
                Ok(value) => {
                    let result = value.json::<Option<DiscordUser>>().await;
                    if let Ok(json) = result {
                        return Msg::Loaded(json.is_some());
                    }
                }
                Err(err) => log!(format!("error {:?}", err)),
            }
            Msg::Loaded(get_false())
        });

        MainPage {
            is_logged_in: false,
            is_admin: false,
            join_code: LobbyId { id: "".to_string() },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Login => {
                // Logic to handle login
                self.is_logged_in = true;
                true
            }
            Msg::Loaded(va) => {
                if va {
                    self.is_logged_in = true
                };
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <style>
                    {"
                        .container {
                            display: flex;
                            justify-content: center; /* Center horizontally */
                            align-items: center; /* Center vertically */
                            height: 100vh; /* Full height of the viewport */
                        }

                        .main-content {
                            text-align: center; /* Center content */
                        }
                    "}
                </style>
                <div class="container">
                    <div class="main-content">
                        <h1>{"Main Page"}</h1>
                        { self.view_login_button() }
                    </div>
                </div>
            </>
        }
    }
}
impl MainPage {
    fn view_login_button(&self) -> Html {
        let onclick = Callback::from(move |_| {
            if let Some(window) = window() {
                if window.location().set_href("discord").is_ok() {
                    return;
                }
            }
            panic!("Failed to redirect to Discord.");
        });

        let input = Callback::from(move |_| {
            if let Some(window) = window() {
                if window.location().set_href("/game/main").is_ok() {
                    return;
                }
            }
            panic!("Failed to redirect to Discord.");
        });

        if self.is_logged_in {
            html! {
                    <>
                        <button>{"Create Game"}</button>
                        <div class="input-group">
                            <button onclick={input}>{"Join"}</button>
                        </div>
                    </>
            }
        } else {
            html! {
                    <>
                        <button onclick={onclick}>{"Login Discord"}</button>
                        <div class="input-group">
                            <button onclick={input}>{"Join"}</button>
                         </div>
                    </>
            }
        }
    }
}

