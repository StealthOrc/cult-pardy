use crate::service::FrontendService;
use cult_common::parse_addr_str;
use futures::channel::mpsc::TrySendError;
use futures::SinkExt;
use gloo_console::{info, log};
use gloo_net::websocket::Message;
use wasm_bindgen::JsValue;
use wasm_cookies::cookies::*;
use web_sys::HtmlDocument;
use yew::prelude::*;

use crate::websocket::WebsocketService;
// testing purposes
fn document() -> HtmlDocument {
    use wasm_bindgen::JsCast;

    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .dyn_into::<HtmlDocument>()
        .unwrap()
}

fn cookie_string() -> String {
    document().cookie().unwrap()
}

pub enum Msg {
    TestMessage(String),
}

pub struct App {
    ws_service: WebsocketService,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let usr_session_id: String = get(&cookie_string(), "user-session-id")
            .expect("could not get cookie")
            .expect("could not get cookie from user");
        let session_token: String = get(&cookie_string(), "session-token")
            .expect("could not get cookie")
            .expect("could not get cookie from user");

        let lobby_id = "main";
        let wss = WebsocketService::new(
            parse_addr_str("127.0.0.1", 8000).to_string().as_str(),
            lobby_id,
            usr_session_id.as_str(),
            session_token.as_str()
        );
        App { ws_service: wss }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log!("We were here update()");
        let ws = self
            .ws_service
            .send_tunnel
            .try_send(Message::Text("Test".parse().unwrap()));
        match ws {
            Ok(e) => {
                info!("OK SEND")
            }
            Err(e) => {
                log!("Hello", e.to_string())
            }
        };

        match msg {
            Msg::TestMessage(test) => todo!("TestMessage: not implemented"),
        };
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let test1 = "100€";
        let test2 = "200€";
        let test3 = "300€";
        let _cultpardy = "cult-pardy";
        let header1 = "🌟 Anime";
        let header2 = "🎨 Art";
        let header3 = "🗻 Japan";
        let header4 = "🎹 Music";
        let header5 = "🍿 Movies and watchables";
        let _onclick = {};
        html! {
        <main>
            <div class="listcontainer">
                <ul>
                  <li>
                      <ul>
                        <h2>{header1}</h2>
                        <div class="button-container"><button>{test1}</button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header2}</h2>
                        <div class="button-container"><button>{test1} </button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header3}</h2>
                        <div class="button-container"><button>{test1}</button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header4}</h2>
                        <div class="button-container"><button>{test1}</button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header5}</h2>
                        <div class="button-container"><button>{test1}</button></div>
                        <div class="button-container"><button>{test2}</button></div>
                        <div class="button-container"><button>{test3}</button></div>
                    </ul>
                  </li>
                </ul>
            </div>
        </main>
        }
    }
}
