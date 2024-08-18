use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ApiError {
    Session(ApiSessionError),
    Request(ApiRequestError),
    Internal(String),

}

pub trait ToResponse {
    fn to_response(&self) -> HttpResponse;
    
}

pub trait ToApiError {
    fn to_api_error(&self) -> ApiError;
    
}

impl ToResponse for ApiError {
     fn to_response(&self) -> HttpResponse {
        match self.clone(){
            ApiError::Session(error) => error.to_response(),
            ApiError::Request(error) =>  error.to_response(),
            ApiError::Internal(_) =>  HttpResponse::InternalServerError().json(self),
        }
    }
    
}





#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ApiSessionError {
    SessionNotFound,
    SessionExpired(String),
    SessionInvalid(String),
    SessionError(String),
    SessionNotAuthorized(String),
    SessionNotAuthenticated(String),
    SessionNotAdmin(String),
}

impl ToResponse for ApiSessionError { 
    fn to_response(&self) -> HttpResponse {
        match self.clone(){
            ApiSessionError::SessionNotFound => HttpResponse::NotFound().json(self),
            ApiSessionError::SessionExpired(_) => HttpResponse::Unauthorized().json(self),
            ApiSessionError::SessionInvalid(_) => HttpResponse::BadRequest().json(self),
            ApiSessionError::SessionError(_) => HttpResponse::InternalServerError().json(self),
            ApiSessionError::SessionNotAuthorized(_) => HttpResponse::Unauthorized().json(self),
            ApiSessionError::SessionNotAuthenticated(_) => HttpResponse::Unauthorized().json(self),
            ApiSessionError::SessionNotAdmin(_) => HttpResponse::Unauthorized().json(self),
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

impl ToResponse for ApiRequestError {
    fn to_response(&self) -> HttpResponse {
        match self.clone(){
            ApiRequestError::RequestError(_,_) => HttpResponse::InternalServerError().json(self),
            ApiRequestError::RequestInvalid(_,_) => HttpResponse::BadRequest().json(self),
            ApiRequestError::RequestNotFound(_) => HttpResponse::NotFound().json(self),
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
