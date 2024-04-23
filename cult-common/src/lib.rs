
use actix_web::{HttpRequest, HttpResponse};
use actix_web::http::header::ToStrError;
use actix_web::web::{Json, JsonConfig};
use serde::{Deserialize, Serialize};
use core::option::Option;
use std::io::BufRead;
use actix_web::cookie::time::format_description::FormatItem::Optional;
use serde_json::Value;

#[derive(Deserialize, Serialize)]
enum HeaderErrorType{
    Missing,
    Empty,
    StringConversion,
}

#[derive(Deserialize, Serialize)]
struct HeaderRequestError{
    header_name: String,
    error_type: HeaderErrorType
}

impl HeaderRequestError {

    fn new(name: &str, error_type:HeaderErrorType) -> Self {
       HeaderRequestError{
            header_name: name.to_string(),
            error_type
        }
    }



}
pub fn get_internal_server_error_json(body:Value) -> HttpResponse {
    HttpResponse::InternalServerError().json(body)
}


pub fn extract_value<'a >(req: &'a HttpRequest, key: &str) -> Option<&'a str> {
    let query_string = req.query_string();
    for pair in query_string.split('&') {
        let mut parts = pair.split('=');
        if let Some(k) = parts.next() {
            if k == key {
                if let Some(v) = parts.next() {
                    return Some(v);
                }
            }
        }
    }
    None
}


pub fn extract_header_string(req: &HttpRequest, header_name: &str) -> Result<String, HttpResponse> {
    match req.headers().get(header_name) {
        None => Err(HttpResponse::InternalServerError().json( HeaderRequestError::new(header_name, HeaderErrorType::Missing))),
        Some(value) => {
            if value.is_empty() {
                Err(HttpResponse::InternalServerError().json( HeaderRequestError::new(header_name, HeaderErrorType::Empty)))
            } else {
                match value.to_str() {
                    Ok(text) => Ok(text.to_string()),
                    Err(_) => Err(HttpResponse::InternalServerError().json( HeaderRequestError::new(header_name, HeaderErrorType::StringConversion))),
                }
            }
        }
    }
}