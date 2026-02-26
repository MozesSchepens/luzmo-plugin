use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
// Centralized error handling for the Luzmo plugin
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub r#type: ErrorType,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorType {
    pub code: u16,
    pub description: String,
}

#[derive(Debug)]
pub struct PluginError {
    pub code: u16,
    pub description: &'static str,
    pub message: String,
}

impl PluginError {
    pub fn new(code: u16, description: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            description,
            message: message.into(),
        }
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}): {}", self.description, self.code, self.message)
    }
}

impl ResponseError for PluginError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            r#type: ErrorType {
                code: self.code,
                description: self.description.to_string(),
            },
            message: self.message.clone(),
        })
    }
}

pub fn http_error(code: u16, description: &'static str, message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .json(ErrorResponse {
            r#type: ErrorType {
                code,
                description: description.to_string(),
            },
            message: message.into(),
        })
}