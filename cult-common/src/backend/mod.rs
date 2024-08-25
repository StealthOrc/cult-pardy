use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize};
use tsify_next::Tsify;
use utoipa::ToSchema;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

use crate::dto::board::{DtoCategory, DtoJeopardyBoard, DtoQuestion};
use crate::wasm_lib::ids::lobby::LobbyId;
use crate::wasm_lib::ids::usersession::UserSessionId;
use crate::wasm_lib::websocket_events::MediaState;
use crate::wasm_lib::{JeopardyMode, Media, MediaType, NumberScope, QuestionType, Vector2D, VideoType};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
pub enum LobbyCreateResponse {
    Created(LobbyId),
    Error(String),
}

#[derive(Tsify,Debug, Clone, Serialize, ToSchema)]
pub struct JeopardyBoard {
    pub title: String,
    pub categories: Vec<Category>,
    #[serde(skip_serializing)]
    #[tsify(optional)]
    pub current: Option<Vector2D>,
    #[serde(skip_serializing)]
    pub create: DateTime<Local>,
    #[serde(skip_serializing)]
    pub action_state: Arc<Mutex<ActionState>>,
}


impl JeopardyBoard {

    pub fn default(mode: JeopardyMode) -> Self {
        let mut categories: Vec<Category> = Vec::new();
        for category in 0..mode.field_size() {
            let category_name = format!("Category_{}", category);
            let mut questions: Vec<Question> = Vec::new();
            let mut value = 100;
            for question in 0..mode.field_size() {
                let mut question_type : QuestionType = QuestionType::Question;
                if question == 0 && category == 0 {
                    question_type = QuestionType::Youtube("dQw4w9WgXcQ".to_string());
                }
                if question == 1 && category == 0 {
                    let media = Media::new(MediaType::Video(vec![VideoType::default()]), "FlyHigh.mp4".to_string());
                    question_type = QuestionType::Media(media);
                }
                if question == 2 && category == 0 {
                    let first_time_slot = NumberScope::new(0, 10);
                    let second_time_slot = NumberScope::new(60, 70);
                    let vec: Vec<NumberScope> = vec![first_time_slot, second_time_slot];
                    let media = Media::new(MediaType::Video(vec![VideoType::time_slots(vec)]), "FlyHigh.mp4".to_string());
                    question_type = QuestionType::Media(media);
                }
                if question == 3 && category == 0 {
                    let media = Media::new(MediaType::Video(vec![VideoType::Slowmotion(30)]), "FlyHigh.mp4".to_string());
                    question_type = QuestionType::Media(media);
                }

                let question_name = format!("question_{}", question);
                let answer_name = format!("answer{}", question);
                let question = Question {
                    value,
                    question: question_name,
                    question_types: vec![question_type],
                    answer: answer_name,
                    open: false,
                    won_user_id: None,
                };
                value = value * 2;
                questions.push(question)
            }

            categories.push(Category::new(category_name, questions))
        }
        JeopardyBoard {
            title: "Default JeopardyBoard".to_string(),
            categories,
            current: None,
            create: Local::now(),
            action_state: Arc::new(Mutex::new(ActionState{state: ActionStateType::None, current_type: None})),
        }
    }

    
    pub fn dto(&self, creator:UserSessionId) -> DtoJeopardyBoard {
        let cat = self
            .categories
            .iter()
            .enumerate()
            .map(|(row_index, category)| {
                let questions = category
                    .questions
                    .iter()
                    .enumerate()
                    .map(|(col_index, question)| {
                        let current: Vector2D = Vector2D {
                            x: row_index,
                            y: col_index,
                        };
                        match self.current {
                        None => question.clone().dto(false, current),
                        Some(vec) =>  question.clone().dto(vec.eq(&current), current)
                        }
                    })
                    .collect::<Vec<DtoQuestion>>();

                DtoCategory {
                    title: category.clone().title,
                    questions,
                }
            })
            .collect::<Vec<DtoCategory>>();

        let current = match self.current {
            None => None,
            Some(vec) => {
                let question = self.get_question(vec).unwrap();
                Some(question.dto(true, vec))
            }
        };
        DtoJeopardyBoard {
            creator,
            categories: cat,
            current,
            action_state: self.action_state.lock().expect("Error while locking action state").clone(),
        }
    }

    pub fn get_question(&self, vector2d: Vector2D) -> Option<Question> {
        if let Some(categories) = self.categories.get(vector2d.x) {
            if let Some(question) = categories.questions.get(vector2d.y) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_mut_question(&mut self, vector2d: Vector2D) ->  Option<&mut Question> {
        if let Some(categories) = self.categories.get_mut(vector2d.x) {
            return categories.questions.get_mut(vector2d.y)
        }
        None
    }

    pub fn get_current(&self) -> Option<Question> {
        if let Some(current) = self.current {
            if let Some(question) = self.get_question(current) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_mut_current(&mut self) -> Option<&mut Question> {
        if let Some(current) = self.current {
         return self.get_mut_question(current);
        }
        None
    }

    pub fn get_value_and_remove_current(&mut self, won_user_id: &UserSessionId) -> Option<i32> {
        let mut value : Option<i32> = None;
        if let Some(current) = self.current {
            if let Some(question) = self.get_mut_question(current) {
                question.open = true;
                question.won_user_id = Some(won_user_id.clone());
                value = Some(question.value);
            }
        }
        self.current = None;
        value
    }



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
            action_state:   Arc::new(Mutex::new(ActionState{state: ActionStateType::None, current_type: None})),
        };

        Ok(board)
    }
}


#[derive(Tsify,Debug, Clone, Serialize, Deserialize)]
pub struct ActionState {
    pub state: ActionStateType,
    pub current_type: Option<QuestionType>,
}





#[derive(Tsify,Debug, Clone, Serialize, Deserialize)]
#[tsify(namespace)] 
pub enum ActionStateType {
    None,
    MediaPlayer(MediaState),
}








impl ActionState {

    pub fn update(&mut self, action: ActionState) {
        *self = action;
    }

    pub fn update_action_state_type(&mut self, action_state_type: ActionStateType) {
        self.state = action_state_type;
    }
    
}


/*#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MediaPlayer {
    pub starting_time: DateTime<Local>,
    pub video_start: i32,
    pub video_end: i32,
    pub restarting_time: Option<DateTime<Local>>,
    pub current: i32,
}

impl MediaPlayer {

    pub fn default() -> Self {
        MediaPlayer {
            starting_time: Local::now(),
            video_start: 0,
            video_end: 0,
            restarting_time: None,
            current: 0,
        }
    }
    
}*/


#[derive(Tsify,Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
pub struct Category {
    pub title: String,
    pub questions: Vec<Question>,
}

impl Category {

    pub fn new(title: String, questions: Vec<Question>) -> Self {
        Category { title, questions }
    }

    pub fn dto(self, x: usize) -> DtoCategory {
        DtoCategory {
            title: self.title,
            questions: self
                .questions
                .iter().enumerate()
                .map(|(index, question)| question.clone().dto(false, Vector2D{x,y : index}))
                .collect(),
        }
    }
}


#[derive(Tsify, Debug, Clone, Serialize, Eq, PartialEq, ToSchema)]
pub struct Question {
    pub question_types: Vec<QuestionType>,
    pub question: String,
    pub value: i32,
    pub answer: String,
    #[serde(skip_serializing)]
    pub open: bool,
    #[serde(skip_serializing)]
    #[tsify(optional)]
    pub won_user_id: Option<UserSessionId>,
}



impl Question {

    pub fn dto(self, current: bool, vector2d: Vector2D) -> DtoQuestion {
        let question_text = match current {
            true => Some(self.question),
            false => None,
        };
        let answer: Option<String> = match self.open {
            true => Some(self.answer),
            false => None,
        };
        DtoQuestion {
            question_types: self.question_types,
            value: self.value,
            question_text,
            answer,
            won_user_id: self.won_user_id,
            vector2d,
        }
    }
}

impl<'de> Deserialize<'de> for Question {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PartialQuestion {
            question_types: Vec<QuestionType>,
            question: String,
            value: i32,
            answer: String,
        }
        let partial_question = PartialQuestion::deserialize(deserializer)?;
        let question = Question {
            question_types: partial_question.question_types,
            question: partial_question.question,
            value: partial_question.value,
            answer: partial_question.answer,
            open: false,
            won_user_id: None,
        };

        Ok(question)
    }
}