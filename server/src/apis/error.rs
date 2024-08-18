use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::{openapi::{RefOr, Response, ResponseBuilder}, ToResponse, ToSchema};

#[allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ApiError {
    Session(ApiSessionError),
    Game(ApiGameError),
    Request(ApiRequestError),
    Internal(String),

}

pub trait ToResponse2 {
    fn to_response(&self) -> HttpResponse;
    
}

pub trait ToApiError {
    fn to_api_error(&self) -> ApiError;
    
}

impl ToResponse2 for ApiError {
     fn to_response(&self) -> HttpResponse {
        match self.clone(){
            ApiError::Session(error) => error.to_response(), 
            ApiError::Request(error) =>  error.to_response(),
            ApiError::Game(error) => error.to_response(),
            ApiError::Internal(_) =>  HttpResponse::InternalServerError().json(self), // 500
        }
    }
    
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, ToResponse)]
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

impl ToResponse2 for ApiGameError {
    fn to_response(&self) -> HttpResponse {
        match self.clone(){
            ApiGameError::GameError(_) => HttpResponse::InternalServerError().json(self), // 500
            ApiGameError::LobbyInvalid(_) => HttpResponse::BadRequest().json(self), // 400
            ApiGameError::LobbyNotFound(_) => HttpResponse::NotFound().json(self), // 404
        }
    }
    
}




#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, ToResponse)]
pub enum ApiSessionError {
    SessionNotFound,
    SessionExpired(String),
    SessionInvalid(String),
    SessionError(String),
    SessionNotAuthorized(String),
    SessionNotAuthenticated(String),
    SessionNotAdmin(String),
}

impl ToResponse2 for ApiSessionError { 
    fn to_response(&self) -> HttpResponse {
        match self.clone(){
            ApiSessionError::SessionNotFound => HttpResponse::NotFound().json(self), // 404
            ApiSessionError::SessionExpired(_) => HttpResponse::Unauthorized().json(self), // 401
            ApiSessionError::SessionInvalid(_) => HttpResponse::BadRequest().json(self), // 400 
            ApiSessionError::SessionError(_) => HttpResponse::InternalServerError().json(self), // 500 
            ApiSessionError::SessionNotAuthorized(_) => HttpResponse::Unauthorized().json(self), // 401 
            ApiSessionError::SessionNotAuthenticated(_) => HttpResponse::Unauthorized().json(self), // 401
            ApiSessionError::SessionNotAdmin(_) => HttpResponse::Unauthorized().json(self), // 401
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
    RequestError(String, ErrorType),
    RequestInvalid(String, ErrorType),
    RequestNotFound(String),
    
}

impl ToResponse2 for ApiRequestError {
    fn to_response(&self) -> HttpResponse {
        match self.clone(){
            ApiRequestError::RequestError(_,_) => HttpResponse::InternalServerError().json(self), // 500
            ApiRequestError::RequestInvalid(_,_) => HttpResponse::BadRequest().json(self), // 400
            ApiRequestError::RequestNotFound(_) => HttpResponse::NotFound().json(self), // 404 
        }
    }
    
}

impl ToApiError for ApiRequestError {
    fn to_api_error(&self) -> ApiError {
        ApiError::Request(self.clone())
    }
}

impl ApiRequestError {
    pub fn new_error(message: String) -> ApiRequestError {
        ApiRequestError::RequestError(message, ErrorType::StringConversion)
    }

    pub fn new_invalid(message: String) -> ApiRequestError {
        ApiRequestError::RequestInvalid(message, ErrorType::StringConversion)
    }

    pub fn new_not_found(message: String) -> ApiRequestError {
        ApiRequestError::RequestNotFound(message)
    }


}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]

pub enum ErrorType {
    Missing,
    Empty,
    StringConversion,
}
