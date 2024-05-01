use std::collections::HashMap;
use std::fmt::Arguments;
use std::net::SocketAddr;
use std::thread::Thread;
use std::time::Duration;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use rand::{rngs::ThreadRng, Rng, random};

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

#[derive(Debug, Clone,Copy, Hash, Eq, PartialEq, Default)]
pub struct UserSessionId {
    pub id:usize,
}


impl Serialize for UserSessionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_u64(self.id as u64)
    }
}

impl<'de> Deserialize<'de> for UserSessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let id_str: String = Deserialize::deserialize(deserializer)?;
        let id = id_str.parse().map_err(serde::de::Error::custom)?;
        println!("test?");
        Ok(UserSessionId { id })
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
            id
        }
    }
    pub fn from_string(id:String) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        UserSessionId{
            id
        }
    }
    pub fn from_str(id:&str) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        UserSessionId{
            id
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
