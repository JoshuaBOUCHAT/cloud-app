use actix_web::{HttpResponse, ResponseError};
use bb8::RunError;

use redis::RedisError;
use serde::Serialize;
use thiserror::Error;

use crate::{
    auth::auth_models::token::TokenError,
    constants::messages::{SERIALIZATION_FAILED, TOKEN_EXPIRED, TOKEN_INVALID},
};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Mail sending error: {0}")]
    Mail(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Forbiden: {0}")]
    Forbiden(String),
}
impl From<sqlx::error::Error> for AppError {
    fn from(value: sqlx::error::Error) -> Self {
        eprintln!("Database error:{value}");
        AppError::Database(
            value
                .as_database_error()
                .and_then(|err| err.code())
                .and_then(|code| Some(code.to_string()))
                .unwrap_or("NOT DB ERROR".to_string()),
        )
    }
}
impl From<RunError<redis::RedisError>> for AppError {
    fn from(value: RunError<redis::RedisError>) -> Self {
        Self::Cache(value.to_string())
    }
}
impl From<RedisError> for AppError {
    fn from(value: RedisError) -> Self {
        Self::Cache(value.to_string())
    }
}
impl From<lettre::transport::smtp::Error> for AppError {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        eprintln!("error:{}", value);
        Self::Mail(value.to_string())
    }
}

impl From<lettre::error::Error> for AppError {
    fn from(value: lettre::error::Error) -> Self {
        eprintln!("error:{}", value);
        Self::Mail(value.to_string())
    }
}
impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        eprintln!("error:{}", value);
        Self::Internal("".to_string())
    }
}
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        eprintln!("Intenal error: {value}");
        Self::Internal(SERIALIZATION_FAILED.to_string())
    }
}
impl From<TokenError> for AppError {
    fn from(value: TokenError) -> Self {
        match value {
            TokenError::Expired => AppError::Unauthorized(TOKEN_EXPIRED.to_string()),
            TokenError::Invalid => AppError::Validation(TOKEN_INVALID.to_string()),
            TokenError::EncodeError(msg) => AppError::Internal(msg),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        HttpResponse::build(status).json(ErrorResponse {
            error: self.to_string(),
            code: status.as_u16(),
        })
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;
        match self {
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Cache(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Mail(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Forbiden(_) => StatusCode::FORBIDDEN,
        }
    }
}
