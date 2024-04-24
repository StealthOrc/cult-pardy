use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::BufRead;
use std::net::SocketAddr;

#[derive(Deserialize, Serialize)]
enum ErrorType {
    Missing,
    Empty,
    StringConversion,
}

#[derive(Deserialize, Serialize)]
struct RequestError {
    header_name: String,
    error_type: ErrorType,
}

impl RequestError {
    fn new(name: &str, error_type: ErrorType) -> Self {
        RequestError {
            header_name: name.to_string(),
            error_type,
        }
    }
}
pub fn get_internal_server_error_json(body: Value) -> HttpResponse {
    HttpResponse::InternalServerError().json(body)
}

pub fn extract_value(req: &HttpRequest, key: &str) -> Result<String, HttpResponse> {
    let query_string = req.query_string();
    if query_string.is_empty() {
        return Err(
            HttpResponse::InternalServerError().json(RequestError::new(key, ErrorType::Empty))
        );
    }
    for pair in query_string.split('&') {
        let mut parts = pair.split('=');
        if let Some(k) = parts.next() {
            if k == key {
                if let Some(v) = parts.next() {
                    return Ok(v.to_string());
                }
            }
        }
    }
    return Err(
        HttpResponse::InternalServerError().json(RequestError::new(key, ErrorType::Missing))
    );
}

pub fn extract_header_string(req: &HttpRequest, header_name: &str) -> Result<String, HttpResponse> {
    match req.headers().get(header_name) {
        None => Err(HttpResponse::InternalServerError()
            .json(RequestError::new(header_name, ErrorType::Missing))),
        Some(value) => {
            if value.is_empty() {
                Err(HttpResponse::InternalServerError()
                    .json(RequestError::new(header_name, ErrorType::Empty)))
            } else {
                match value.to_str() {
                    Ok(text) => Ok(text.to_string()),
                    Err(_) => Err(HttpResponse::InternalServerError()
                        .json(RequestError::new(header_name, ErrorType::StringConversion))),
                }
            }
        }
    }
}

pub fn parse_addr_str(domain: &str, port: usize) -> SocketAddr {
    let addr = "127.0.0.1";
    let port = 8081;
    let addr = format!("{}:{}", addr, port);
    let addr = addr.parse::<SocketAddr>().expect("Failed to parse address");
    addr
}
