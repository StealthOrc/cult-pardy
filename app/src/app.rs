use cult_common::{
    parse_addr_str, BoardEvent, DtoJeopardyBoard, DtoQuestion, Vector2D, WebsocketServerEvents,
    WebsocketSessionEvent,
};
use futures::StreamExt;
use gloo_console::{info, log};
use gloo_net::websocket::{self, Message};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    rc::Rc,
};
use wasm_cookies::cookies::*;
use web_sys::HtmlDocument;
use yew::prelude::*;

use crate::ws::websocket::WebsocketService;

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

pub(crate) fn cookie_string() -> String {
    document().cookie().unwrap()
}

pub enum Msg {
    TestMessage(String),
    SendWebSocket,
}

#[derive(Properties, Clone, PartialEq, Default)]
pub struct ButtonProps {
    #[prop_or_default]
    pub dtoq: DtoQuestion,
    #[prop_or_default]
    pub vec: Vector2D,
    #[prop_or_default]
    pub name: String,
}

#[function_component]
fn Button(props: &ButtonProps) -> Html {
    let p = props.clone();
    let onclick = Callback::from(move |_| {
        log!(format!("{:?}", p.vec));
    });
    if props.dtoq.won_user_id.is_some() {
        html! {
        <button {onclick}>{format!("Disabled: {}",props.name) }</button>
        }
    } else {
        html! {
        <div class="button-container"><button onclick={onclick}>{props.name.clone()}</button></div>
        }
    }
}
type SharedStateDtoJeopardyBoard = Rc<RefCell<Option<DtoJeopardyBoard>>>;

pub struct App {
    ws_service: WebsocketService,
    jp_board_dto: SharedStateDtoJeopardyBoard,
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

        let lobby_id = get_game_id_from_url().expect("SomeData?");

        // type SharedStateDtoJeopardyBoard = Rc<RefCell<Option<DtoJeopardyBoard>>>;
        let mut testString: Rc<RefCell<String>> = Rc::new(RefCell::new(String::from("Test")));
        let mut dto: SharedStateDtoJeopardyBoard = Rc::new(RefCell::new(None));
        let on_read = {
            let mydto: SharedStateDtoJeopardyBoard = Rc::clone(&dto);
            let mytestString = Rc::clone(&testString);
            move |event: WebsocketServerEvents| match event {
                WebsocketServerEvents::Board(board_event) => match board_event {
                    BoardEvent::CurrentBoard(board) => {
                        // let mut test = (*mydto).borrow_mut().unwrap();
                        // test = board;
                        (*mydto).replace(Some(board));
                        (*mytestString).replace(String::from("test2"));

                        log!("------");
                        log!(format!("onread:{:?}", (*mydto).borrow().as_ref()));
                    }
                    BoardEvent::UpdateBoard(_) => {}
                },
                WebsocketServerEvents::Websocket(_) => {}
                WebsocketServerEvents::Session(_) => {}
                WebsocketServerEvents::Error(_) => {}
                WebsocketServerEvents::Text(_) => {}
            }
        };

        let dto = dto;
        log!(format!("{:?}", (*dto).borrow().as_ref()));

        let wss = WebsocketService::new(
            parse_addr_str("127.0.0.1", 8000).to_string().as_str(),
            lobby_id.as_str(),
            usr_session_id.as_str(),
            session_token.as_str(),
            on_read,
        );

        App {
            ws_service: wss,
            jp_board_dto: dto,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TestMessage(test) => {
                log!(format!("update(): {test}"));
                true
            }
            Msg::SendWebSocket => {
                let ws = self
                    .ws_service
                    .send_tunnel
                    .try_send(Message::Text("Test".parse().unwrap()));
                match ws {
                    Ok(_) => {
                        log!("OK SEND")
                    }
                    Err(e) => {
                        log!("Hello", e.to_string())
                    }
                };
                true
            }
            _ => true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        log!("------");
        log!(format!(
            "view(): {:?}",
            (*(self.jp_board_dto)).borrow().as_ref()
        ));

        let test1 = "100€";
        let test2 = "200€";
        let test3 = "300€";
        let _cultpardy = "cult-pardy";
        let header1 = "🌟 Anime";
        let header2 = "🎨 Art";
        let header3 = "🗻 Japan";
        let header4 = "🎹 Music";
        let header5 = "🍿 Movies and watchables";
        // let testcb = ctx.link().callback(Msg::TestMessage);
        // testcb.emit("Hello World!".to_owned());
        // let onclick = ctx
        //     .link()
        //     .callback(|_| Msg::TestMessage(String::from("clicked")));
        // #[prop_or_default]
        // pub dtoq: DtoQuestion,
        // #[prop_or_default]
        // pub vec: Vector2D,
        // #[prop_or_default]
        // pub id: usize,
        let vec2d = Vector2D { x: 1, y: 2 };
        html! {
        <main>
            <div class="listcontainer">
                <ul>
                  <li>
                      <ul>
                        <h2>{header1}</h2>
                        <Button name={test1} vec={vec2d.clone()}/>
                        <Button name={test2} vec={vec2d.clone()}/>
                        <Button name={test3} vec={vec2d.clone()}/>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header2}</h2>
                        <Button name={test1} vec={vec2d.clone()}/>
                        <Button name={test2} vec={vec2d.clone()}/>
                        <Button name={test3} vec={vec2d.clone()}/>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header3}</h2>
                        <Button name={test1} vec={vec2d.clone()}/>
                        <Button name={test2} vec={vec2d.clone()}/>
                        <Button name={test3} vec={vec2d.clone()}/>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header4}</h2>
                        <Button name={test1} vec={vec2d.clone()}/>
                        <Button name={test2} vec={vec2d.clone()}/>
                        <Button name={test3} vec={vec2d.clone()}/>
                    </ul>
                  </li>
                  <li>
                      <ul>
                        <h2>{header5}</h2>
                        <Button name={test1} vec={vec2d.clone()}/>
                        <Button name={test2} vec={vec2d.clone()}/>
                        <Button name={test3} vec={vec2d.clone()}/>
                    </ul>
                  </li>
                </ul>
            </div>
        </main>
        }
    }
}
pub(crate) fn get_game_id_from_url() -> Option<String> {
    let window = web_sys::window().expect("No global `window` exists.");
    let location = window.location();
    let pathname = location
        .pathname()
        .expect("Failed to get pathname from URL");
    let parts: Vec<&str> = pathname.split('/').collect();
    parts.last().map(|&s| s.to_string())
}
