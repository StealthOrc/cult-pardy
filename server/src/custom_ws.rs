use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use actix_web_actors::ws::Message;
use actix_web_actors::ws::Message::{Binary,  Text};

pub struct GameWS;

impl Actor for GameWS {
    type Context = ws::WebsocketContext<Self>;
}
impl StreamHandler<anyhow::Result<Message, ws::ProtocolError>> for GameWS {
    fn handle(&mut self, msg: anyhow::Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Text(text)) => {
                let respone = match text.to_string().as_str() {
                    "ping" => "Pong",
                    _ => &text,
                };
                ctx.text(respone)
            },
            Ok(Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}




