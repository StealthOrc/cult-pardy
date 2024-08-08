
use chrono::{DateTime, Duration, Local};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use std::hash::{Hash};

use crate::dto::api::DTOFileToken;




#[derive(Tsify, Debug, Clone, Hash, Deserialize, Serialize,Eq, PartialEq, Default)]
pub struct FileToken {
    pub token: String,
    pub expires: DateTime<Local>,
}


impl FileToken {
    pub fn new() -> FileToken {
        let token = Self::new_token();
        FileToken {
            token,
            expires: Local::now() + Duration::hours(1),
        }
    }

    pub fn random() -> FileToken {
        let token = Self::new_token();
        FileToken {
            token,
            expires: Local::now() + Duration::hours(1),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires < Local::now()
    }

    fn new_token() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect()
    }

    pub fn to_dto_file_token(&self) -> DTOFileToken {
        DTOFileToken {
            token: self.token.clone(),
        }
    }
}