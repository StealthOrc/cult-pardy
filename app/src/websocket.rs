use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use wasm_bindgen_futures::spawn_local;

pub struct WebsocketService {
    pub send_tunnel: Sender<Message>,
}
impl WebsocketService {
    // add code here
    pub fn new(addr: &str, lobby_id: &str, user_session_id: &str) -> Self {
        let ws = WebSocket::open(
            format!("ws://{addr}/ws?lobby-id={lobby_id}&user-session-id={user_session_id}")
                .as_str(),
        )
        .unwrap();

        let (mut write, mut read) = ws.split();

        let (tunnel_send, mut tunnel_receive) = futures::channel::mpsc::channel::<Message>(1000);

        spawn_local(async move {
            while let Some(msg) = tunnel_receive.next().await {
                match msg {
                    Message::Text(data) => {
                        println!("sending Text:{:?}", data);
                        write.send(Message::Text(data)).await.unwrap();
                    }
                    Message::Bytes(b) => {
                        println!("sending Bytes:{:?}", b);
                        write.send(Message::Bytes(b)).await.unwrap();
                    }
                }
            }
        });

        spawn_local(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(data)) => {
                        println!("from websocket {}", data);
                    }
                    Ok(Message::Bytes(b)) => {
                        let decoded = std::str::from_utf8(&b);
                        if let Ok(val) = decoded {
                            println!("from websocket (bin){}", val);
                        }
                    }
                    Err(e) => {
                        println!("ws error:{}", e);
                    }
                }
            }
        });

        Self {
            send_tunnel: tunnel_send,
        }
    }
}
