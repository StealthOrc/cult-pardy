
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use std::hash::Hash;
use wasm_bindgen::prelude::*;

use crate::backend::{ActionState, BuzzerState};
use crate::wasm_lib::ids::usersession::UserSessionId;
use crate::wasm_lib::{DiscordUser, QuestionType, Vector2D};




#[derive(Tsify,Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct DTOSession {
    pub user_session_id: UserSessionId,
    pub score: i32,
    #[tsify(optional)]
    pub discord_user: Option<DiscordUser>,
    pub is_admin: bool,
}

impl DTOSession {
    pub fn user_session_id(self) -> UserSessionId {
        self.user_session_id
    }
    pub fn discord_user(self) -> Option<DiscordUser> {
        self.discord_user
    }

}

#[derive(Tsify,Debug, Clone, Serialize, Deserialize)]
pub struct DtoJeopardyBoard {
    pub creator: UserSessionId,
    pub categories: Vec<DtoCategory>,
    #[tsify(optional)]
    pub current: Option<DtoQuestion>,
    pub action_state: ActionState,
    pub buzzer_state: BuzzerState,
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

    pub fn get_mut_question(&mut self, vector2d: Vector2D) -> Option<DtoQuestion> {
        if let Some(categories) = self.categories.get_mut(vector2d.x) {
            if let Some(question) = categories.questions.get_mut(vector2d.y) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_current(&self) -> Option<DtoQuestion> {
        return self.current.clone();
    }

    pub fn get_mut_current(&mut self) -> Option<DtoQuestion> {
        let opt = self.current.clone();
        if let Some(current) = opt {
            if let Some(question) = self.get_mut_question(current.vector2d) {
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
    #[tsify(optional)]
    pub question_text: Option<String>,
    pub value: i32,
    #[tsify(optional)]
    pub answer: Option<String>,
    #[tsify(optional)]
    pub won_user_id: Option<UserSessionId>,
    pub vector2d: Vector2D,
}
