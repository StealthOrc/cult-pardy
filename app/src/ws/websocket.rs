use cult_common::{compress, decompress, WebsocketServerEvents, WS_PROTOCOL};

use crate::types::AppMsg;
use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use gloo_console::log;
use gloo_net::websocket::{futures::WebSocket, Message};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone)]
pub struct WebsocketService {
    pub send_tunnel: Sender<Message>,
}

impl PartialEq for WebsocketService {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl WebsocketService {
    // add code here
    pub fn new(
        addr: &str,
        lobby_id: &str,
        user_session_id: &str,
        session_token: &str,
        callback: Callback<WebsocketServerEvents>,
    ) -> Self {
        let ws = WebSocket::open(
            format!("{WS_PROTOCOL}{addr}/ws?lobby-id={lobby_id}&user-session-id={user_session_id}&session-token={session_token}")
                .as_str(),
        )
            .unwrap();

        let (mut write, mut read) = ws.split();
        let (tunnel_send, mut tunnel_receive) = futures::channel::mpsc::channel::<Message>(1000);

        //spawn receiving thread
        spawn_local(async move {
            while let Some(msg) = tunnel_receive.next().await {
                match msg {
                    Message::Text(data) => {
                        log!("sending Text:{:?}", JsValue::from(data.clone()));
                        write.send(Message::Text(data)).await.unwrap();
                    }
                    Message::Bytes(b) => {
                        let bytes = compress(&b).expect("could not compress bytes");
                        log!(format!("sending Bytes:{:?}", bytes));
                        write.send(Message::Bytes(bytes)).await.unwrap();
                    }
                }
            }
        });

        //spawn sending thread
        spawn_local(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(data)) => {
                        println!("from websocket {}", data);
                    }
                    Ok(Message::Bytes(data)) => {
                        if let Ok(bytes) = decompress(&data) {
                            match serde_json::from_slice::<WebsocketServerEvents>(&bytes) {
                                Ok(event) => {
                                    callback.emit(event);
                                }
                                Err(err) => {
                                    log!(
                                        "Error deserializing JSON data: {}",
                                        JsValue::from(err.to_string())
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log!("ws error:{}", JsValue::from(e.to_string()));
                    }
                }
            }
        });

        Self {
            send_tunnel: tunnel_send,
        }
    }
}
