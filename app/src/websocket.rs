use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use wasm_bindgen_futures::spawn_local;

pub struct WebsocketService {
    pub send_tunnel: Sender<String>,
}
impl WebsocketService {
    // add code here
    pub fn new(addr: &str) -> Self {
        let ws = WebSocket::open(format!("ws://{}/ws", addr).as_str()).unwrap();
        let (mut write, mut read) = ws.split();

        let (tunnel_send, mut tunnel_receive) = futures::channel::mpsc::channel::<String>(1000);

        spawn_local(async move {
            while let Some(s) = tunnel_receive.next().await {
                println!("message from channel {}", s);
                write.send(Message::Text(s)).await.unwrap();
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
