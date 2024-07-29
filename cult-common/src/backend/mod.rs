use chrono::{DateTime, Local};
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use rand::distributions::Alphanumeric;
use rand::{random, Rng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tsify_next::Tsify;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::string::ToString;
use strum::Display;
use wasm_bindgen::prelude::*;

use crate::dto::{DtoCategory, DtoJeopardyBoard, DtoQuestion};
use crate::wasm_lib::ids::lobby::LobbyId;
use crate::wasm_lib::ids::usersession::UserSessionId;
use crate::wasm_lib::{JeopardyMode, QuestionType, Vector2D};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum LobbyCreateResponse {
    Created(LobbyId),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct JeopardyBoard {
    pub title: String,
    pub categories: Vec<Category>,
    #[serde(skip_serializing)]
    pub current: Option<Vector2D>,
    #[serde(skip_serializing)]
    pub create: DateTime<Local>,
}


impl JeopardyBoard {
    pub fn default(mode: JeopardyMode) -> Self {
        let mut categories: Vec<Category> = Vec::new();
        for category in 0..mode.field_size() {
            let category_name = format!("Category_{}", category);
            let mut questions: Vec<Question> = Vec::new();
            let mut value = 100;
            for question in 0..mode.field_size() {
                let question_name = format!("question_{}", question);
                let answer_name = format!("answer{}", question);
                let question = Question {
                    value,
                    question: question_name,
                    question_type: QuestionType::Question,
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
        }
    }

    pub fn dto(self, creator:UserSessionId) -> DtoJeopardyBoard {
        let cat = self
            .categories
            .iter()
            .enumerate()
            .map(|(row_index, category)| {
                let questions = category
                    .questions
                    .iter()
                    .enumerate()
                    .map(|(col_index, question)| match self.current {
                        None => question.clone().dto(false),
                        Some(vec) => {
                            let current = Vector2D {
                                x: row_index,
                                y: col_index,
                            };
                            question.clone().dto(vec.eq(&current))
                        }
                    })
                    .collect::<Vec<DtoQuestion>>();

                DtoCategory {
                    title: category.clone().title,
                    questions,
                }
            })
            .collect::<Vec<DtoCategory>>();

        DtoJeopardyBoard {
            creator,
            categories: cat,
            current: self.current,
        }
    }

    pub fn get_question(self, vector2d: Vector2D) -> Option<Question> {
        if let Some(categories) = self.categories.get(vector2d.x) {
            if let Some(question) = categories.questions.get(vector2d.y) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_mut_question(&mut self, vector2d: Vector2D) -> Option<Question> {
        if let Some(categories) = self.categories.get_mut(vector2d.x) {
            if let Some(question) = categories.questions.get_mut(vector2d.y) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_current(self) -> Option<Question> {
        if let Some(current) = self.current {
            if let Some(question) = self.get_question(current) {
                return Some(question.clone());
            }
        }
        None
    }

    pub fn get_mut_current(&mut self) -> Option<Question> {
        if let Some(current) = self.current {
            if let Some(question) = self.get_mut_question(current) {
                return Some(question.clone());
            }
        }
        None
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
        };

        Ok(board)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Category {
    pub title: String,
    pub questions: Vec<Question>,
}

impl Category {

    pub fn new(title: String, questions: Vec<Question>) -> Self {
        Category { title, questions }
    }

    pub fn dto(self) -> DtoCategory {
        DtoCategory {
            title: self.title,
            questions: self
                .questions
                .into_iter()
                .map(|question| question.dto(false))
                .collect(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct Question {
    pub question_type: QuestionType,
    pub question: String,
    pub value: i32,
    pub answer: String,
    #[serde(skip_serializing)]
    pub open: bool,
    #[serde(skip_serializing)]
    pub won_user_id: Option<UserSessionId>,
}
impl Question {

    pub fn dto(self, current: bool) -> DtoQuestion {
        let question_text = match current {
            true => Some(self.question),
            false => None,
        };
        let answer = match self.open {
            true => Some(self.answer),
            false => None,
        };
        DtoQuestion {
            question_type: self.question_type,
            value: self.value,
            question_text,
            answer,
            won_user_id: self.won_user_id,
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