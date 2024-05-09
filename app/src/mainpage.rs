use std::fmt::format;
use wasm_cookies::cookies::get;
use web_sys::{window};
use yew::{html, Callback, Component, Html, Context, Properties};

use yew_router::prelude::RouterScopeExt;
use cult_common::{LobbyId, LOCATION, PROTOCOL};
use crate::app;

#[derive(Properties, PartialEq)]
pub struct MainPage {
    is_logged_in: bool,
    is_admin: bool,
    join_code: LobbyId,
}

pub enum Msg {
    Login,
    CreateGame,
}

impl Component for MainPage {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let _request_url = format!("{}/api/discord_session", format!("{}{}",PROTOCOL,LOCATION));
        let _usr_session_id: String = get(&app::cookie_string(), "user-session-id")
            .expect("could not get cookie")
            .expect("could not get cookie from user");
        let _session_token: String = get(&app::cookie_string(), "session-token")
            .expect("could not get cookie")
            .expect("could not get cookie from user");

        let _cookies = vec![
            ("usr-session-id", "1"),
            ("session-token", "2"),
        ];

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
            }
            Msg::CreateGame => {
                // Logic to handle creating a game
            }
        }
        true
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
                    if let Ok(_) = window.location().set_href("discord") {
                        return ();
                }
            }
            panic!("Failed to redirect to Discord.");
        });

        let input = Callback::from(move |_| {
            if let Some(window) = window() {
                if let Ok(_) = window.location().set_href("/game/main") {
                    return ();
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