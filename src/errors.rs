use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

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
pub enum PluginError {
    UnknownColumn { message: String },
    UnsupportedAggregation { message: String },
    DatasetNotFound { message: String },
    InvalidRequest { message: String },
    InternalError { message: String },
}

impl PluginError {
    fn meta(&self) -> (StatusCode, &'static str, &str) {
        match self {
            PluginError::UnknownColumn { message } => (StatusCode::BAD_REQUEST, "Unknown column", message),
            PluginError::UnsupportedAggregation { message } => (StatusCode::BAD_REQUEST, "Unsupported aggregation", message),
            PluginError::DatasetNotFound { message } => (StatusCode::NOT_FOUND, "Unknown dataset", message),
            PluginError::InvalidRequest { message } => (StatusCode::BAD_REQUEST, "Invalid request", message),
            PluginError::InternalError { message } => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error", message),
        }
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (status, desc, msg) = self.meta();
        write!(f, "{} ({}): {}", desc, status.as_u16(), msg)
    }
}

impl ResponseError for PluginError {
    fn status_code(&self) -> StatusCode {
        self.meta().0
    }

    fn error_response(&self) -> HttpResponse {
        let (status, description, message) = self.meta();
        HttpResponse::build(status).json(ErrorResponse {
            r#type: ErrorType {
                code: status.as_u16(),
                description: description.to_string(),
            },
            message: message.to_string(),
        })
    }
}