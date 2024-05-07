use cult_common::{
    compress, parse_addr_str, BoardEvent, DtoJeopardyBoard, DtoQuestion, Vector2D,
    WebsocketServerEvents,
};
use futures::StreamExt;
use gloo_console::log;
use gloo_net::websocket::Message;
use std::{borrow::Borrow, cell::RefCell, rc::Rc};
use wasm_cookies::cookies::*;
use web_sys::HtmlDocument;
use yew::prelude::*;

use crate::types::AppMsg;
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

#[derive(Properties, Clone, PartialEq, Default)]
pub struct ButtonProps {
    #[prop_or_default]
    pub dtoq: DtoQuestion,
    #[prop_or_default]
    pub name: String,
    pub onclick: Callback<MouseEvent>,
}

#[function_component]
fn Button(props: &ButtonProps) -> Html {
    if props.dtoq.won_user_id.is_some() {
        html! {
        <button onclick={props.onclick.clone()}>{format!("Disabled: {}",props.name) }</button>
        }
    } else {
        html! {
        <div class="button-container"><button onclick={props.onclick.clone()}>{props.name.clone()}</button></div>
        }
    }
}
type SharedStateDtoJeopardyBoard = Rc<RefCell<Option<DtoJeopardyBoard>>>;

pub struct App {
    ws_service: WebsocketService,
    jp_board_dto: SharedStateDtoJeopardyBoard,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let usr_session_id: String = get(&cookie_string(), "user-session-id")
            .expect("could not get cookie")
            .expect("could not get cookie from user");
        let session_token: String = get(&cookie_string(), "session-token")
            .expect("could not get cookie")
            .expect("could not get cookie from user");

        let lobby_id = get_game_id_from_url().expect("SomeData?");

        // type SharedStateDtoJeopardyBoard = Rc<RefCell<Option<DtoJeopardyBoard>>>;
        let dto: SharedStateDtoJeopardyBoard = Rc::new(RefCell::new(None));
        let on_read = {
            let mydto: SharedStateDtoJeopardyBoard = Rc::clone(&dto);
            move |event: WebsocketServerEvents, callback: Callback<AppMsg>| match event {
                WebsocketServerEvents::Board(board_event) => match board_event {
                    BoardEvent::CurrentBoard(board) => {
                        log!("board recieved!");
                        (*mydto).replace(Some(board));
                        callback.emit(AppMsg::BoardLoaded);
                    }
                    BoardEvent::UpdateBoard(_) => {}
                },
                WebsocketServerEvents::Websocket(_) => {}
                WebsocketServerEvents::Session(_) => {}
                WebsocketServerEvents::Error(_) => {}
                WebsocketServerEvents::Text(_) => {}
            }
        };

        log!(format!("create(): dto={:?}", (*dto).borrow().as_ref()));

        let wss = WebsocketService::new(
            parse_addr_str("127.0.0.1", 8000).to_string().as_str(),
            lobby_id.as_str(),
            usr_session_id.as_str(),
            session_token.as_str(),
            on_read,
            ctx.link().callback(|msg: AppMsg| msg),
        );

        App {
            ws_service: wss,
            jp_board_dto: dto,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::BoardUnloaded => {
                log!("Board is not yet loaded!");
                false
            }
            AppMsg::BoardLoaded => {
                log!("Board was loaded, lets refresh!");
                true
            }
            AppMsg::GetButtonQuestion(buttonpos) => {
                log!(format!("GetButtonQuestion: {:?}", buttonpos));
                let eventdata = cult_common::WebsocketSessionEvent::Click(buttonpos);
                let binary = serde_json::to_vec(&eventdata).expect("Can´t convert to vec");
                let ws = self.ws_service.send_tunnel.try_send(Message::Bytes(binary));
                match ws {
                    Ok(_) => {
                        log!("SedWebSocket: OK SEND")
                    }
                    Err(e) => {
                        log!("SedWebSocket: Error: ", e.to_string())
                    }
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let board = (*(self.jp_board_dto)).borrow().as_ref().cloned();
        log!(format!("view() {:?}", board));
        match board {
            None => {
                log!("sending msg: Msg::BoardUnloaded");
                ctx.link().send_message(AppMsg::BoardUnloaded);
                html! {
                    <h1>{ "LOADING..." }</h1>
                }
            }
            Some(board) => get_board(&board, ctx),
        }
    }
}

pub fn get_board(dto_jeopardy_board: &DtoJeopardyBoard, ctx: &Context<App>) -> Html {
    let num_categories = dto_jeopardy_board.categories.len();

    let grid_columns = format!("repeat({}, 1fr)", num_categories);

    html! {
        <main class="jeopardy-container">
            <div class="jeopardy-board" style={format!("grid-template-columns: {}", grid_columns)}>
                {
                    dto_jeopardy_board.categories.iter().enumerate().map(|(row_index, category)| {
                        html! {
                            <div class="jeopardy-category">
                                <h2>{&category.title}</h2>
                                {
                                    category.questions.iter().enumerate().map(|(col_index, question)| {
                                        let vec2D = Vector2D { x: row_index as u8, y: col_index as u8 };
                                        let onclick = ctx.link().callback(move |_| {
                                            AppMsg::GetButtonQuestion(vec2D)
                                        });
                                        html! {
                                            <div class="jeopardy-question">
                                                <Button dtoq={question.clone()} name={format!("{}€", question.value)} {onclick} />
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
