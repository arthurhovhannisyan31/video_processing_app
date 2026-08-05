use axum::extract::multipart::MultipartError;
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::Error;
use std::io;
use std::net::AddrParseError;
use std::num::ParseIntError;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum DomainError {
  #[error("Access is forbidden")]
  Forbidden,
  #[error("Invalid credentials")]
  InvalidCredentials,
  #[error("internal error: {0}")]
  Internal(String),
  #[error("User already exists")]
  UserAlreadyExists,
  #[error("User not found: {0}")]
  UserNotFound(i64),
  #[error("Validation failed: {0}")]
  Validation(String),
}

#[derive(Debug, Error)]
pub enum ApplicationError {
  #[error("Bad request: {0}")]
  BadRequest(String),
  #[error("Conflict: {0}")]
  Conflict(String),
  #[error("Forbidden")]
  Forbidden,
  #[error("Internal server error: {0}")]
  Internal(String),
  #[error("Not found: {0}")]
  NotFound(String),
  #[error("Unauthorized")]
  Unauthorized,
  #[error("validation error: {0}")]
  Validation(String),
}

#[derive(Debug, Error)]
pub enum ServerError {
  #[error("Parse addr error")]
  AddrParseError(#[from] AddrParseError),
  #[error("IO Error")]
  IO(#[from] io::Error),
  #[error("Parse int error")]
  ParseIntError(#[from] ParseIntError),
  #[error("Sqlx error: {0}")]
  SqlxError(String),
  #[error("Failed to read env variable: {0}")]
  VarError(String),
  #[error("Failed loading .env file")]
  Dotenv(#[from] dotenvy::Error),
  #[error(transparent)]
  OtherError(#[from] anyhow::Error),
}

impl From<sqlx::Error> for ServerError {
  fn from(value: Error) -> Self {
    ServerError::SqlxError(value.to_string())
  }
}

impl IntoResponse for ApplicationError {
  fn into_response(self) -> Response {
    match self {
      ApplicationError::BadRequest(msg) => {
        (StatusCode::BAD_REQUEST, json!({"message": msg}).to_string())
          .into_response()
      }
      ApplicationError::Conflict(msg) => {
        (StatusCode::CONFLICT, json!({"message": msg}).to_string())
          .into_response()
      }
      ApplicationError::Forbidden => StatusCode::FORBIDDEN.into_response(),
      ApplicationError::Internal(msg) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": msg}).to_string(),
      )
        .into_response(),
      ApplicationError::NotFound(msg) => {
        (StatusCode::NOT_FOUND, json!({"message": msg}).to_string())
          .into_response()
      }
      ApplicationError::Unauthorized => {
        StatusCode::UNAUTHORIZED.into_response()
      }
      ApplicationError::Validation(msg) => {
        (StatusCode::BAD_REQUEST, json!({"message": msg}).to_string())
          .into_response()
      }
    }
  }
}

impl From<DomainError> for ApplicationError {
  fn from(value: DomainError) -> Self {
    match value {
      DomainError::Forbidden => ApplicationError::Forbidden,
      DomainError::InvalidCredentials => ApplicationError::Unauthorized,
      DomainError::Internal(msg) => ApplicationError::Internal(msg),
      DomainError::UserAlreadyExists => {
        ApplicationError::Conflict("User already exists".to_string())
      }
      DomainError::UserNotFound(id) => {
        ApplicationError::NotFound(format!("User not found: {}", id))
      }
      DomainError::Validation(msg) => ApplicationError::Validation(msg),
    }
  }
}

impl From<ValidationErrors> for ApplicationError {
  fn from(value: ValidationErrors) -> Self {
    ApplicationError::BadRequest(format!(r"{value}"))
  }
}

impl From<MultipartError> for ApplicationError {
  fn from(err: MultipartError) -> Self {
    ApplicationError::BadRequest(err.to_string())
  }
}
