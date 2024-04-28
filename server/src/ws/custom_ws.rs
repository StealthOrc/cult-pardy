use std::collections::HashMap;
use actix::{Actor, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws;
use actix_web_actors::ws::Message;
use actix_web_actors::ws::Message::{Binary,  Text};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};

pub struct GameWS;

#[derive(Serialize, Deserialize, Debug)]
struct Dto {
    id: u32,
    r#type: String,
}


impl Actor for GameWS {
    type Context = ws::WebsocketContext<Self>;
}
impl StreamHandler<anyhow::Result<Message, ws::ProtocolError>> for GameWS {
    fn handle(&mut self, msg: anyhow::Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Text(text)) => {
                let s = text.to_string();
                let dto : Dto = serde_json::from_str(s.as_str()).expect("Test");
                println!("{:?}", dto);
                ctx.text(GameState::new().get_game_state());
            },
            Ok(Binary(bin)) => {
                println!("Binary");

                match bin {

                    Bytes { .. } => {
                    }

                }
            },
            _ => (),
        }
    }
}




#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    value: i32,
    question_text: String,
    answer: String,
}
#[derive(Debug, Serialize, Deserialize)]
enum GameType {
    GameData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    gametype: GameType,
    categories: HashMap<String, Vec<Question>>,
    lobby_id: String,
}


impl GameState {
    fn new() -> Self {
        let mut categories = HashMap::new();
        categories.insert(
            "Pokemom".to_string(),
            vec![
                Question {
                    value: 100,
                    question_text: "Welche Farbe hat dito".to_string(),
                    answer: "grün".to_string(),
                },
                Question {
                    value: 21,
                    question_text: "Welche Farbe hat doto".to_string(),
                    answer: "gelb".to_string(),
                },
            ],
        );
        categories.insert(
            "Pokemom2".to_string(),
            vec![
                Question {
                    value: 100,
                    question_text: "Welche Farbe hat dito2".to_string(),
                    answer: "grün".to_string(),
                },
                Question {
                    value: 21,
                    question_text: "Welche Farbe hat doto2".to_string(),
                    answer: "gelb".to_string(),
                },
            ],
        );
        GameState {
            gametype: GameType::GameData, categories,
            lobby_id : thread_rng().sample_iter(Alphanumeric).take(5).map(char::from).collect()
        }
    }


    fn get_game_state(&self) -> String {
        serde_json::to_string(&self).expect("TODO: panic message")
    }
    #[allow(dead_code)]
    fn get_question(&self, category: &str, value: i32) -> Option<&Question> {
        if let Some(questions) = self.categories.get(category) {
            for question in questions {
                if question.value == value {
                    return Some(question);
                }
            }
        }
        None
    }
}

