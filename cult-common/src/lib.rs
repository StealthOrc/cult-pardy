use std::collections::{HashMap, HashSet};
use std::fmt::Arguments;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::thread::Thread;
use std::time::{Duration, Instant, SystemTime};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use rand::{rngs::ThreadRng, Rng, random};
use rand::distributions::Alphanumeric;
use serde::de::Visitor;
use chrono::{DateTime, Local, TimeZone};
use rand::seq::index::IndexVec;
use serde_json::from_str;
use strum::{Display, EnumIter};


pub fn parse_addr_str(domain: &str, port: usize) -> SocketAddr {
    let addr = format!("{}:{}", domain, port);
    let addr = addr.parse::<SocketAddr>().expect("Failed to parse address");
    addr
}



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub avatar_id: String,
    pub discriminator: String,
    pub global_name: String,
}

impl DiscordUser {
    pub fn avatar_image_url(self) -> String {
        format!("https://cdn.discordapp.com/avatars/{}/{}.jpg",self.id,self.avatar_id)
    }

    pub fn discord_id(self) -> DiscordID {
        DiscordID::new(self.id)
    }



}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct DTOSession{
    pub user_session_id:UserSessionId,
    pub discord_user:Option<DiscordUser>
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

#[derive(Debug, Clone,Serialize, Eq, PartialEq)]
pub enum LobbyCreateResponse{
    Created(LobbyId),
    Error(String),
}




#[derive(Debug, Clone,Serialize, Eq, PartialEq)]
pub struct JeopardyBoard {
    pub title: String,
    pub categories: Vec<Category>,
    #[serde(skip_serializing)]
    pub current: Option<Vector2D>,
    #[serde(skip_serializing)]
    pub create: DateTime<Local>
}


impl<'de> Deserialize<'de> for JeopardyBoard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PartialJeopardyBoard {
            title: String,
            categories: Vec<Category>,
        }

        let partial_board = PartialJeopardyBoard::deserialize(deserializer)?;

        let board = JeopardyBoard {
            title: partial_board.title,
            categories: partial_board.categories,
            current: None,
            create: Local::now(),
        };

        Ok(board)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtoJeopardyBoard {
    pub categories: Vec<DtoCategory>,
    pub current: Option<Vector2D>
}


#[derive(Debug,Clone, Serialize, Deserialize,Eq, PartialEq)]
pub struct Vector2D {
    pub x: u8,
    pub y: u8,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtoCategory {
    pub title: String,
    pub questions: Vec<DtoQuestion>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtoQuestion {
    pub question_type: QuestionType,
    pub question_text: Option<String>,
    pub value: i32,
    pub answer: Option<String>,
    pub won_user_id: Option<UserSessionId>,
}



impl crate::DtoCategory {
    pub fn new(title:String, questions:Vec<DtoQuestion>) -> Self{
        crate::DtoCategory {
            title,
            questions,
        }
    }

}



#[derive(Debug, Clone,Serialize, Deserialize,Eq, PartialEq)]
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

    pub fn dto(self) -> DtoCategory {
        DtoCategory {
            title: self.title,
            questions: self.questions.into_iter().map(|question| question.dto()).collect(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct UserSessionId {
    pub id:String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct DiscordID {
    pub id:String,
}

impl DiscordID {
    pub fn new(id:String) -> Self{
        DiscordID{
            id
        }
    }

    pub fn of_str(id:&str) -> Self{
        DiscordID{
            id:id.to_string()
        }
    }

    pub fn server() -> Self {
        DiscordID{
            id:"000000000000000".to_string()
        }
    }

}


#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
}

impl ApiResponse {

    pub fn new(success:bool) -> Self {
        ApiResponse{
            success,
        }
    }
    pub fn of(success:bool) -> Self {
        ApiResponse{
            success,
        }
    }
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
    pub fn id(self) -> usize {
        return self.id.parse::<usize>().unwrap();
    }

    pub fn of(id:usize) -> Self{
        UserSessionId{
            id:id.to_string(),
        }
    }
    pub fn from_string(id:String) -> Self{
        UserSessionId{
            id,
        }
    }
    pub fn from_str(id:&str) -> Self{
        UserSessionId{
            id:id.to_string(),
        }
    }

    pub fn random() -> Self {
        UserSessionId {
            id: random::<usize>().to_string(),
        }
    }
}






#[derive(Debug,  Clone,Serialize, Eq, PartialEq)]
pub struct Question {
    pub question_type: QuestionType,
    pub question: String,
    pub value: i32,
    pub answer: String,
    #[serde(skip_serializing)]
    pub open: bool,
    #[serde(skip_serializing)]
    pub won_user_id: Option<UserSessionId>
}


impl<'de> Deserialize<'de> for Question {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PartialQuestion {
            question_type: QuestionType,
            question: String,
            value: i32,
            answer: String,
        }
        let partial_question = PartialQuestion::deserialize(deserializer)?;
        let question = Question {
            question_type: partial_question.question_type,
            question: partial_question.question,
            value: partial_question.value,
            answer: partial_question.answer,
            open: false,
            won_user_id: None,
        };

        Ok(question)
    }
}





#[derive(Debug, Clone,Serialize, Deserialize,Eq, PartialEq)]
pub enum  QuestionType {
    Media(String),
    Question,
}








impl Question {
    pub fn dto(self) -> DtoQuestion{
        let question_text=  match self.open {
            true => Some(self.question),
            false => None
        };
        let answer=  match self.open {
            true => Some(self.answer),
            false => None
        };
        DtoQuestion{
            question_type: self.question_type,
            value: self.value,
            question_text,
            answer,
            won_user_id: self.won_user_id,
        }

    }

}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum WebsocketServerEvents {
    Board(BoardEvent),
    Websocket(WebsocketEvent),
    Session(SessionEvent),
    Error(WebsocketError),
    Text(String)
}

impl WebsocketServerEvents{

    pub fn event_name(self) -> String {
        let wse = self.to_string();
        let event= match self {
            WebsocketServerEvents::Board(event) => event.to_string(),
            WebsocketServerEvents::Websocket(event) => event.to_string(),
            WebsocketServerEvents::Session(event) => event.to_string(),
            WebsocketServerEvents::Error(event) => event.to_string(),
            WebsocketServerEvents::Text(event) => event.to_string(),
        };

        format!("{} -> {} ", wse, event)
    }



}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum BoardEvent {
    CurrentBoard(DtoJeopardyBoard),
    UpdateBoard(String),
}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum WebsocketEvent {
    WebsocketJoined(WebsocketSessionId),
    WebsocketDisconnected(WebsocketSessionId),
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, Hash)]
pub enum SessionEvent {
    CurrentSessions(Vec<DTOSession>),
    SessionJoined(UserSessionId),
    SessionDisconnected(UserSessionId),
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum WebsocketSessionEvent {
    Text(String),
    Click(Vector2D),
}


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum WebsocketError{
    LobbyNotFound(LobbyId),
    SessionNotFound(UserSessionId),
    GameStarted(LobbyId),
    NotAuthorized,
    WebsocketCrashed,
    UNKNOWN(String),
}


#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize,Deserialize)]
pub struct WebsocketSessionId {
    id:String,
}


impl WebsocketSessionId{


    pub fn id(self) -> usize {
        return self.id.parse::<usize>().unwrap();
    }

    pub fn random() ->Self  {
        WebsocketSessionId{
            id: random::<usize>().to_string()
        }
    }
    pub fn of(id:usize) -> Self{
        WebsocketSessionId{
            id:id.to_string()
        }
    }
    pub fn from_string(id:String) -> Self{
        WebsocketSessionId{
            id: id.to_string()
        }
    }
    pub fn from_str(id:&str) -> Self{
        let id=  id.parse::<usize>().expect("Can´t convert String to usize");
        WebsocketSessionId{
            id:id.to_string()
        }
    }
}




#[derive(Debug, Clone,  Hash, Eq, PartialEq)]
pub struct LobbyId{
    pub id: String,
}

impl Serialize for LobbyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.id)
    }
}

impl<'de> Deserialize<'de> for LobbyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let id: String = Deserialize::deserialize(deserializer)?;
        Ok(LobbyId { id })
    }
}


impl LobbyId{
    pub fn of(id:String) -> Self{
        LobbyId{
            id,
        }
    }

    pub fn from_str(id:&str) -> Self{
        LobbyId{
            id:id.to_string(),
        }
    }

    pub fn random() -> Self {
        let id = rand::thread_rng().sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from).collect();
        LobbyId{
            id,
        }

    }
}


impl JeopardyBoard {
    pub fn default(mode: JeopardyMode) -> Self {
        let mut categories:Vec<Category > = Vec::new();
        for category in 0..mode.field_size() {
            let category_name = format!("Category_{}", category);
            let mut questions: Vec<Question> = Vec::new();
            let mut value = 100;
            for question in 0..mode.field_size() {
                let question_name = format!("question_{}", question);
                let answer_name = format!("answer{}", question);
                let question = Question{
                    value,
                    question: question_name,
                    question_type: QuestionType::Question,
                    answer: answer_name,
                    open: false,
                    won_user_id: None,
                };
                value = value*2;
                questions.push(question)
            }

            categories.push(Category::new(category_name, questions))
        }
        JeopardyBoard{
            title: "Default JeopardyBoard".to_string(),
            categories,
            current: None,
            create: Local::now(),
        }
    }

    pub fn dto(mut self) -> DtoJeopardyBoard {
        let current = match self.current {
            None => None,
            Some(question) => Some(question)
        };
        DtoJeopardyBoard{
            categories: self.categories.into_iter().map(|category| category.dto()).collect(),
            current,
        }

    }

}

pub fn get_false() -> bool {
    false
}

pub fn get_true() -> bool {
    true
}
