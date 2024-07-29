use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use std::hash::{Hash, Hasher};
use std::string::ToString;
use wasm_bindgen::prelude::*;

use crate::wasm_lib::ids::usersession::UserSessionId;
use crate::wasm_lib::{DiscordUser, QuestionType, Vector2D};



#[derive(Tsify,Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[wasm_bindgen]
pub struct DTOSession {
    #[wasm_bindgen(skip)]
    pub user_session_id: UserSessionId,
    pub score: i32,
    #[wasm_bindgen(skip)]
    pub discord_user: Option<DiscordUser>,
    pub is_admin: bool,
}

#[wasm_bindgen]
impl DTOSession {
    #[wasm_bindgen(getter)]
    pub fn user_session_id(self) -> UserSessionId {
        self.user_session_id
    }
    #[wasm_bindgen(getter)]
    pub fn discord_user(self) -> Option<DiscordUser> {
        self.discord_user
    }

}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DtoJeopardyBoard {
    pub creator: UserSessionId,
    pub categories: Vec<DtoCategory>,
    pub current: Option<Vector2D>,
}

impl DtoJeopardyBoard {
    pub fn get_question(self, vector2d: Vector2D) -> Option<DtoQuestion> {
        if let Some(categories) = self.categories.get(vector2d.x) {
            if let Some(question) = categories.questions.get(vector2d.y) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_mut_question(mut self, vector2d: Vector2D) -> Option<DtoQuestion> {
        if let Some(categories) = self.categories.get_mut(vector2d.x) {
            if let Some(question) = categories.questions.get_mut(vector2d.y) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_current(self) -> Option<DtoQuestion> {
        if let Some(current) = self.current {
            if let Some(question) = self.get_question(current) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_mut_current(self) -> Option<DtoQuestion> {
        if let Some(current) = self.current {
            if let Some(question) = self.get_mut_question(current) {
                return Some(question.clone());
            }
        }
        None
    }
}


#[derive(Tsify,Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DtoCategory {
    pub title: String,
    pub questions: Vec<DtoQuestion>,
}

impl DtoCategory {
    pub fn new(title: String, questions: Vec<DtoQuestion>) -> Self {
        DtoCategory { title, questions }
    }
}



#[derive(Tsify,PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default)]
pub struct DtoQuestion {
    pub question_type: QuestionType,
    pub question_text: Option<String>,
    pub value: i32,
    pub answer: Option<String>,
    pub won_user_id: Option<UserSessionId>,
}