use std::net::SocketAddr;
use serde::{Deserialize, Serialize};

pub fn parse_addr_str(domain: &str, port: usize) -> SocketAddr {
    let addr = format!("{}:{}", domain, port);
    let addr = addr.parse::<SocketAddr>().expect("Failed to parse address");
    addr
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserSessionRequest{
    pub session_id : usize
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