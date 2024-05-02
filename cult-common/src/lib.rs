use std::collections::HashMap;
use std::fmt::Arguments;
use std::net::SocketAddr;
use std::thread::Thread;
use std::time::{Duration, Instant, SystemTime};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use rand::{rngs::ThreadRng, Rng, random};
use rand::distributions::Alphanumeric;
use serde::de::Visitor;
use chrono::{DateTime, Local, TimeZone};


pub fn parse_addr_str(domain: &str, port: usize) -> SocketAddr {
    let addr = format!("{}:{}", domain, port);
    let addr = addr.parse::<SocketAddr>().expect("Failed to parse address");
    addr
}



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub avatar_id: String,
    pub discriminator: String,
    pub global_name: String,
}

impl DiscordUser {
    fn avatar_image_url(self) -> String {
        format!("https://cdn.discordapp.com/avatars/{}/{}.jpg",self.id,self.avatar_id)
    }
}






#[derive(Clone, Copy)]
pub enum JeopardyMode{
    //3x3
    SHORT,
    //5x5
    NORMAL,
    //7x7
    LONG
}
impl JeopardyMode {
    pub fn field_size(self) -> usize {
        match self {
            JeopardyMode::SHORT => 3,
            JeopardyMode::NORMAL => 5,
            JeopardyMode::LONG => 7
        }
    }
}



#[derive(Debug, Serialize, Deserialize)]
pub struct JeopardyBoard {
    pub categories: Vec<Category>

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
    pub questions: Vec<Question>
}

impl Category {
    pub fn new(title:String, questions:Vec<Question>) -> Self{
        Category{
            title,
            questions,
        }
    }

}

#[derive(Debug, Clone,Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserSessionId {
    pub id:usize,
}
#[derive(Debug, Clone,Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionToken {
    pub token: String,
    pub create: DateTime<Local>,
}

impl SessionToken {
    pub fn new() -> SessionToken {
       let token=  Self::new_token();
        SessionToken {
            token,
            create: Local::now(),
        }
    }

    pub fn random() -> SessionToken {
        let token=  Self::new_token();
        SessionToken {
            token,
            create: Local::now(),
        }
    }

    pub fn update(&mut self) {
        self.create = Local::now();
        self.token = Self::new_token();
    }
    fn new_token() -> String {
        rand::thread_rng().sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from).collect()
    }

}


#[derive(Debug, Clone,Serialize,Deserialize, Eq, PartialEq, Default)]
pub struct JsonPrinter{
    pub results: HashMap<String, bool>
}

impl JsonPrinter {
    pub fn new() -> Self {
        JsonPrinter{
            results: HashMap::new(),
        }
    }

    pub fn add_string(&mut self, text:String, result:bool) {
        self.results.insert(text, result);
    }

    pub fn add(&mut self, text:&str, result:bool) {
        self.results.insert(text.to_string(), result);
    }

}


impl UserSessionId{
    pub fn of(id:usize) -> Self{
        UserSessionId{
            id,
        }
    }
    pub fn from_string(id:String) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        UserSessionId{
            id,
        }
    }
    pub fn from_str(id:&str) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        UserSessionId{
            id,
        }
    }

    pub fn random() -> Self {
        UserSessionId {
            id: random::<usize>(),
        }
    }
}






#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    value: i32,
    question_text: String,
    answer: String,
}
impl JeopardyBoard {
    pub fn default(mode: JeopardyMode) -> Self {
        let mut categories:Vec<Category > = Vec::new();
        for category in 0..mode.field_size() {
            let category_name = format!("Category_{}", category);
            let mut questions: Vec<Question> = Vec::new();
            for question in 0..mode.field_size() {
                let question_name = format!("question_{}", question);
                let answer_name = format!("answer{}", question);
                let question = Question{
                    value: 0,
                    question_text: question_name,
                    answer: answer_name,
                };
                questions.push(question)
            }

            categories.push(Category::new(category_name, questions))
        }
        JeopardyBoard{
            categories,
        }
    }

}

pub fn get_false() -> bool {
    false
}

pub fn get_true() -> bool {
    true
}
