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
use yew::html::Scope;
use yew::prelude::*;

use crate::types::AppMsg;
use crate::ws::websocket::WebsocketService;
use crate::{board::*, boardbutton::*, boardquestion::*};

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

type SharedStateDtoJeopardyBoard = Rc<RefCell<Option<DtoJeopardyBoard>>>;

#[derive(PartialEq)]
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
            move |event: WebsocketServerEvents, callback: Callback<AppMsg>| {
                log!(format!("Event received -> {}", event.clone().event_name()));
                match event {
                    WebsocketServerEvents::Board(board_event) => match board_event {
                        BoardEvent::CurrentBoard(board) => {
                            log!("board received!");
                            mydto.borrow_mut().replace(board);
                            callback.emit(AppMsg::BoardLoaded);
                        }
                        BoardEvent::CurrentQuestion(vector2d, dto_question) => {
                            match mydto.borrow_mut().as_mut() {
                                Some(board) => {
                                    //got new data, so replace our current model with the new data
                                    board.current = Some(vector2d);
                                    let mut cat =
                                        board.categories.get_mut(vector2d.x as usize).expect(
                                            format!(
                                                "could not get category {} as mutable.",
                                                vector2d.x
                                            )
                                            .as_str(),
                                        );

                                    std::mem::replace(
                                        &mut cat.questions[vector2d.y as usize],
                                        dto_question,
                                    );
                                    callback.emit(AppMsg::ShowQuestion);
                                }
                                None => todo!(),
                            }
                        }
                    },
                    WebsocketServerEvents::Websocket(_) => {}
                    WebsocketServerEvents::Session(_) => {}
                    WebsocketServerEvents::Error(_) => {}
                    WebsocketServerEvents::Text(_) => {}
                }
            }
        };

        log!(format!(
            "create(): dto={:?}",
            (*Rc::clone(&dto)).borrow().as_ref()
        ));

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
            AppMsg::SendWebsocketMessage(event) => {
                log!(format!("GetButtonQuestion: {:?}", event));
                let binary = serde_json::to_vec(&event).expect("Can´t convert to vec");
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
            AppMsg::ShowQuestion => true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let board = (*(self.jp_board_dto)).borrow().as_ref().cloned();
        log!(format!("view() {:?}", board));
        let board = match board {
            None => {
                log!("sending msg: Msg::BoardUnloaded");
                ctx.link().send_message(AppMsg::BoardUnloaded);
                return html! {
                    <h1>{ "LOADING..." }</h1>
                };
            }
            Some(board) => board,
        };
        match board.current {
            Some(current) => {
                let onclick = ctx.link().callback(AppMsg::SendWebsocketMessage);
                let question =
                    board.categories[current.x as usize].questions[current.y as usize].clone();
                html! {
                    <BoardQuestion {question} {onclick}/>
                }
            }
            None => {
                let onclick = ctx.link().callback(AppMsg::SendWebsocketMessage);
                html! {
                    <Board board={board} {onclick}/>
                }
            }
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
