use std::net::SocketAddr;
use yew::prelude::*;
use cult_common::parse_addr_str;

use crate::websocket::WebsocketService;

pub enum Msg {
    TestMessage(String),
}

pub struct App {
    test_data: String,
    ws_service: WebsocketService,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let wss = WebsocketService::new(parse_addr_str("127.0.0.1", 8081).to_string().as_str());
        App {
            test_data: String::from("Test"),
            ws_service: wss,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TestMessage(test) => self.test_data = test,
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