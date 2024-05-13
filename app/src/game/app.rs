use cult_common::{compress, parse_addr_str, BoardEvent, DtoJeopardyBoard, DtoQuestion, UserSessionId, Vector2D, WebsocketServerEvents, LOCATION, DTOSession};
use futures::StreamExt;
use gloo_console::log;
use gloo_net::websocket::Message;
use std::collections::HashMap;
use std::{borrow::Borrow, cell::RefCell, rc::Rc};
use ritelinked::LinkedHashMap;
use wasm_cookies::cookies::*;
use web_sys::HtmlDocument;
use yew::html::Scope;
use yew::prelude::*;
use crate::game::board::Board;
use crate::game::boardquestion::BoardQuestion;
use crate::game::playerlistpanel::PlayerListPanel;

use crate::types::{AppMsg, UserList};
use crate::ws::eventhandler::handleEvent;
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

#[derive(PartialEq)]
pub struct App {
    pub ws_service: WebsocketService,
    pub jp_board_dto: Option<DtoJeopardyBoard>,
    pub user_list: LinkedHashMap<UserSessionId, DTOSession>,
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

        let wss = WebsocketService::new(
            LOCATION,
            lobby_id.as_str(),
            usr_session_id.as_str(),
            session_token.as_str(),
            ctx.link()
                .callback(|event: WebsocketServerEvents| AppMsg::HandleWebsocketEvent(event)),
        );

        App {
            ws_service: wss,
            jp_board_dto: None,
            user_list: LinkedHashMap::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
            AppMsg::HandleWebsocketEvent(event) => handleEvent(self, event),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let board = self.jp_board_dto.clone();
        let user_list = self.user_list.clone();
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
                let cb_add_user_score = ctx.link().callback(AppMsg::SendWebsocketMessage);
                let question = board.categories[current.x].questions[current.y].clone();
                let creator = board.creator;
                html! {
                    <div>
                        <BoardQuestion {question} {onclick}/>
                        <PlayerListPanel {creator} {user_list} add_user_score={cb_add_user_score}/>
                    </div>
                }
            }
            None => {
                let onclick = ctx.link().callback(AppMsg::SendWebsocketMessage);
                let creator = board.creator.clone();
                log!(format!("CREATOR{:?}", creator));
                html! {
                    <div>
                        <Board board={board} {onclick}/>
                        <PlayerListPanel {creator} {user_list} />
                    </div>
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
