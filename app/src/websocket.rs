use std::io::Read;
use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use futures::channel::mpsc::Receiver;
use gloo_console::log;
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_net::websocket::Message::Text;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone)]
pub struct WebsocketService {
    pub send_tunnel: Sender<Message>,
}
impl WebsocketService {
    // add code here
    pub fn new(addr: &str, lobby_id: &str, user_session_id: &str, session_token: &str) -> Self {
        let ws = WebSocket::open(
            format!("ws://{addr}/ws?lobby-id={lobby_id}&user-session-id={user_session_id}&session-token={session_token}")
                .as_str(),
        )
        .unwrap();

        let (mut write, mut read) = ws.split();

        let (tunnel_send, mut tunnel_receive) = futures::channel::mpsc::channel::<Message>(1000);

        spawn_local(async move {
            while let Some(msg) = tunnel_receive.next().await {
                match msg {
                    Message::Text(data) => {
                        log!("sending Text:{:?}", JsValue::from(data.clone()));
                        write.send(Message::Text(data)).await.unwrap();
                    }
                    Message::Bytes(b) => {
                        let mut test = "".to_string();
                        b.as_slice().read_to_string(&mut test).expect("TODO: panic message");
                        log!("sending Bytes:{:?}",JsValue::from(test));
                        write.send(Message::Bytes(b)).await.unwrap();
                    }
                }
            }
        });

        spawn_local(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(data)) => {
                        log!("from websocket {}",JsValue::from(data.clone()));
                    }
                    Ok(Message::Bytes(b)) => {
                        let decoded = std::str::from_utf8(&b);
                        if let Ok(val) = decoded {
                            log!("from websocket (bin){}",JsValue::from(val.clone()));
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
