use crate::domain::error::DomainError;
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

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
      DomainError::PostNotFound(id) => {
        ApplicationError::NotFound(format!("Post not found: {}", id))
      }
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
