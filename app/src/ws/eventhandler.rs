use crate::types::AppMsg;
use cult_common::{BoardEvent, SessionEvent, WebsocketServerEvents};
use gloo_console::log;
use ritelinked::LinkedHashMap;
use serde::de::Unexpected::Option;
use yew::Callback;
use crate::game::app::App;

pub fn handleEvent(app: &mut App, event: WebsocketServerEvents) -> bool {
    log!(format!("Event received -> {}", event.clone().event_name()));
    match event {
        WebsocketServerEvents::Board(event) => handle_board(app, event),
        WebsocketServerEvents::Websocket(_) => false,
        WebsocketServerEvents::Session(event) => handle_session(app, event),
        WebsocketServerEvents::Error(_) => false,
        WebsocketServerEvents::Text(_) => false,
    }
}

fn handle_board(mut app: &mut App, board_event: BoardEvent) -> bool {
    match board_event {
        BoardEvent::CurrentBoard(board) => {
            log!("board received!");
            app.jp_board_dto = Some(board);
            return true;
        }
        BoardEvent::CurrentQuestion(vector2d, dto_question) => match &mut app.jp_board_dto {
            Some(board) => {
                board.current = Some(vector2d);
                let mut cat = board
                    .categories
                    .get_mut(vector2d.x)
                    .expect(format!("could not get category {} as mutable.", vector2d.x).as_str());

                let _ = std::mem::replace(&mut cat.questions[vector2d.y], dto_question);
                return true;
            }
            None => todo!(),
        },
        BoardEvent::UpdateCurrentQuestion(_) => {false},
        BoardEvent::UpdateSessionScore(_,_) => false
    }
}

fn handle_session(mut app: &mut App, session_event: SessionEvent) -> bool {
    match session_event {
        SessionEvent::CurrentSessions(session_vec) => {
            let mut session = LinkedHashMap::new();
            for dto_session in session_vec {
                session.insert(dto_session.user_session_id.clone(), dto_session);
            }
            app.user_list = session;
            true
        }
        SessionEvent::SessionJoined(session) => {
            app.user_list
                .insert(session.user_session_id.clone(), session);
            true
        }
        SessionEvent::SessionDisconnected(session_id) => {
            app.user_list.remove(&session_id);
            true
        }
    }
}
