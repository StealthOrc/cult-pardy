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
    BoardUnloaded
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
        let mut dto: SharedStateDtoJeopardyBoard = Rc::new(RefCell::new(None));
        let on_read = {
            let mydto: SharedStateDtoJeopardyBoard = Rc::clone(&dto);
            move |event: WebsocketServerEvents| match event {
                WebsocketServerEvents::Board(board_event) => match board_event {
                    BoardEvent::CurrentBoard(board) => {
                        (*mydto).replace(Some(board));
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
            Msg::BoardUnloaded => {
                true
            }
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

            _ => false,
        }
    }

    /*     <li>
                                  <ul>
                                    <h2>{header1}</h2>
                                    <Button name={test1} vec={vec2d.clone()}/>
                                    <Button name={test2} vec={vec2d.clone()}/>
                                    <Button name={test3} vec={vec2d.clone()}/>
                                </ul>*/

    fn view(&self, ctx: &Context<Self>) -> Html {
        let board = (*(self.jp_board_dto)).borrow().as_ref().cloned();
        match board {
            None => {
                //ctx.link().send_message(Msg::BoardUnloaded);
                let onclick = ctx.link().callback(|_| Msg::BoardUnloaded);
                return html! {
                    <button {onclick}>{ "LOADING..." }</button>
                }
            },
            Some(board) => {
                get_board(&board)
            }
        }
    }
}


pub fn get_board(dto_jeopardy_board: &DtoJeopardyBoard) -> Html {
    let num_categories = dto_jeopardy_board.categories.len();

    let grid_columns = format!("repeat({}, 1fr)", num_categories);

    return html! {
        <main class="jeopardy-container">
            <div class="jeopardy-board" style={format!("grid-template-columns: {}", grid_columns)}>
                {
                    dto_jeopardy_board.categories.iter().enumerate().map(|(row_index, category)| {
                        html! {
                            <div class="jeopardy-category">
                                <h2>{&category.title}</h2>
                                {
                                    category.questions.iter().enumerate().map(|(col_index, question)| {
                                        html! {
                                            <div class="jeopardy-question">
                                                <Button dtoq={question.clone()} name={format!("{}€", question.value)}  vec={Vector2D { x: row_index as u8, y: col_index as u8 }} />
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </main>
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