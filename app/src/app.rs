use cult_common::parse_addr_str;
use gloo_storage::Storage;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::websocket::WebsocketService;

pub enum Msg {
    TestMessage(String),
}

pub struct App {
    test_data: String,
    ws_service: Option<WebsocketService>,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[cfg(not(debug_assertions))]
    #[at("/game/:id")]
    Game,
    // in debug mode, for testing with trunk serve
    #[cfg(debug_assertions)]
    #[at("/assets/game/:id")]
    Game,
    #[not_found]
    #[at("/404")]
    NotFound,
}

impl App {
    fn setup_ws_connection(&mut self) {
        let usr_session_id: String =
            gloo_storage::SessionStorage::get("user_session_id").unwrap_or_default();
        let lobby_id = "main";
        let wss = WebsocketService::new(
            parse_addr_str("127.0.0.1", 8000).to_string().as_str(),
            lobby_id,
            usr_session_id.as_str(),
        );
        self.ws_service = Some(wss);
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => todo!(),
        Route::Game => game_route(),
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

fn game_route() -> Html {
    // TODO: Somehow call setup_ws_connection, dont know how yet tho...
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

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App {
            test_data: String::from("Test"),
            ws_service: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TestMessage(test) => self.test_data = test,
        };
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        }
    }
}
