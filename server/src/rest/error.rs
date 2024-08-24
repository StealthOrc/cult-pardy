use actix_web::{http, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::{openapi::{security::Http, RefOr, Response, ResponseBuilder}, ToSchema};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ApiError {
    Session(ApiSessionError),
    Game(ApiGameError),
    Request(ApiRequestError),
    Internal(String),
    File(ApiFileError),

}

pub trait ToResponse {
    fn to_response(&self) -> HttpResponse;
    fn to_status_code(&self) -> http::StatusCode;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ApiFileError {
    FileError(String),
    FileInvalid(String),
    FileNotFound(String),
    FileExists,
}

impl ToApiError for ApiFileError {
    fn to_api_error(&self) -> ApiError {
        ApiError::File(self.clone())
    }
    
}

impl ToResponse for ApiFileError {
    fn to_response(&self) -> HttpResponse {
        HttpResponse::build(self.to_status_code()).json(self)
    }

    fn to_status_code(&self) -> http::StatusCode {
        match self.clone(){
            ApiFileError::FileError(_) => http::StatusCode::INTERNAL_SERVER_ERROR, // 500
            ApiFileError::FileInvalid(_) => http::StatusCode::BAD_REQUEST, // 400
            ApiFileError::FileNotFound(_) => http::StatusCode::NOT_FOUND, // 404
            ApiFileError::FileExists => http::StatusCode::CONFLICT, // 409
        }
    }
}



pub trait ToApiError {
    fn to_api_error(&self) -> ApiError;
    
}

impl ToResponse for ApiError {
     fn to_response(&self) -> HttpResponse {
        match self.clone(){
            ApiError::Session(error) => error.to_response(), 
            ApiError::Request(error) =>  error.to_response(),
            ApiError::Game(error) => error.to_response(),
            ApiError::File(error) => error.to_response(),
            ApiError::Internal(_) =>  HttpResponse::InternalServerError().json(self), // 500
        }
    }

    fn to_status_code(&self) -> http::StatusCode {
        match self.clone(){
            ApiError::Session(error) => error.to_status_code(), 
            ApiError::Request(error) =>  error.to_status_code(),
            ApiError::Game(error) => error.to_status_code(),
            ApiError::File(error) => error.to_status_code(),
            ApiError::Internal(_) => http::StatusCode::INTERNAL_SERVER_ERROR, // 500
        }
    }
    
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ApiGameError {
    GameError(String),
    LobbyInvalid(String),
    LobbyNotFound(String),
}

impl ToApiError for ApiGameError {
    fn to_api_error(&self) -> ApiError {
        ApiError::Game(self.clone())
    }
    
}

impl ToResponse for ApiGameError {

    fn to_response(&self) -> HttpResponse {
        HttpResponse::build(self.to_status_code()).json(self)
    }

    fn to_status_code(&self) -> http::StatusCode {
        match self.clone(){
            ApiGameError::GameError(_) => http::StatusCode::INTERNAL_SERVER_ERROR, // 500
            ApiGameError::LobbyInvalid(_) => http::StatusCode::BAD_REQUEST, // 400
            ApiGameError::LobbyNotFound(_) => http::StatusCode::NOT_FOUND, // 404
        }
    }
    


    
}




#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ApiSessionError {
    NotFound,
    Expired,
    Invalid(String),
    Error(String),
    NoDiscordData,
    NotAuthorized,
    NotAuthenticated,
    NotAdmin,
}

impl ToResponse for ApiSessionError {



    fn to_response(&self) -> HttpResponse {
        HttpResponse::build(self.to_status_code()).json(self)
    }


    fn to_status_code(&self) -> http::StatusCode {
        match self.clone(){
            ApiSessionError::NotFound => http::StatusCode::NOT_FOUND, // 404
            ApiSessionError::Expired => http::StatusCode::UNAUTHORIZED, // 401
            ApiSessionError::Invalid(_) => http::StatusCode::BAD_REQUEST, // 400 
            ApiSessionError::Error(_)=> http::StatusCode::INTERNAL_SERVER_ERROR, // 500 
            ApiSessionError::NoDiscordData => http::StatusCode::FORBIDDEN, // 400
            ApiSessionError::NotAuthorized => http::StatusCode::UNAUTHORIZED, // 401 
            ApiSessionError::NotAuthenticated => http::StatusCode::UNAUTHORIZED, // 401
            ApiSessionError::NotAdmin=> http::StatusCode::UNAUTHORIZED, // 401
        }
    }
    

    
    
}

impl ToApiError for ApiSessionError {
    fn to_api_error(&self) -> ApiError {
        ApiError::Session(self.clone())
    }
    
}





#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ApiRequestError {
    Error(String, ErrorType),
    Invalid(String, ErrorType),
    NotFound(String),
}



impl ToApiError for ApiRequestError {
    fn to_api_error(&self) -> ApiError {
        ApiError::Request(self.clone())
    }
}


impl ToResponse for ApiRequestError {

    fn to_response(&self) -> HttpResponse {
        HttpResponse::build(self.to_status_code()).json(self)
    }


    fn to_status_code(&self) -> http::StatusCode {
        match self.clone(){
            ApiRequestError::Error(_,_) => http::StatusCode::INTERNAL_SERVER_ERROR, // 500
            ApiRequestError::Invalid(_,_) => http::StatusCode::BAD_REQUEST, // 400
            ApiRequestError::NotFound(_) => http::StatusCode::NOT_FOUND, // 404 
        }
    }
    
}






#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]

pub enum ErrorType {
    Default,
    Missing,
    Empty,
    StringConversion,
}
