use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::migrate::MigrateError;
use std::io;
use std::net::AddrParseError;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;
use validator::ValidationErrors;

/* Domain objects errors */
#[derive(Debug, Error)]
#[from(sqlx::Error)]
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
  #[error("Media data not found: {0}")]
  MissingMediaData(String),
  #[error("Sqlx error")]
  SqlxError(#[from] sqlx::Error),
}

/* API errors */
#[derive(Debug, Error)]
pub enum ApplicationError {
  #[error("Bad request: {0}")]
  BadRequest(String),
  #[error("Unauthorized")]
  Unauthorized,
  #[error("Forbidden")]
  Forbidden,
  #[error("Not found: {0}")]
  NotFound(String),
  #[error("Conflict: {0}")]
  Conflict(String),
  #[error("Internal server error: {0}")]
  Internal(String),
  #[error("Validation error")]
  Validation(#[from] ValidationErrors),
}

/* Server errors */
#[derive(Debug, Error)]
pub enum ServerError {
  #[error("Parse addr error: {0}")]
  AddrParseError(#[from] AddrParseError),
  #[error("IO Error: {0}")]
  IO(#[from] io::Error),
  #[error("Parse int error: {0}")]
  ParseIntError(#[from] ParseIntError),
  #[error("Parse int error: {0}")]
  ParseFloatError(#[from] ParseFloatError),
  #[error("Sqlx error: {0}")]
  SqlxError(#[from] sqlx::Error),
  #[error("Database migration error: {0}")]
  MigrateError(#[from] MigrateError),
  #[error("Failed to read env variable: {0}")]
  VarError(String),
  #[error("Failed loading .env file: {0}")]
  Dotenv(#[from] dotenvy::Error),
  #[error("Tokio task error: {0}")]
  TokioTaskJoinError(#[from] tokio::task::JoinError),
  #[error("Multipart form error: {0}")]
  Multipart(#[from] MultipartError),
  #[error("Data error: {0}")]
  DataError(String),
  #[error("Processing error: {0}")]
  Processing(String),
  #[error("Serde_json error: {0}")]
  SerdeJson(#[from] serde_json::Error),
  #[error("JWT error: {0}")]
  Jwt(#[from] jsonwebtoken::errors::Error),
  #[error("password hash error: {0}")]
  PasswordHash(#[from] argon2::password_hash::Error),
  #[error(transparent)]
  OtherError(#[from] anyhow::Error),
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
      ApplicationError::Validation(err) => (
        StatusCode::BAD_REQUEST,
        json!({"message": err.to_string()}).to_string(),
      )
        .into_response(),
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
      DomainError::MissingMediaData(msg) => ApplicationError::Internal(msg),
      DomainError::SqlxError(msg) => {
        ApplicationError::Internal(msg.to_string())
      }
    }
  }
}

impl From<ServerError> for ApplicationError {
  fn from(value: ServerError) -> Self {
    match value {
      ServerError::AddrParseError(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::IO(err) => ApplicationError::Internal(err.to_string()),
      ServerError::ParseIntError(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::ParseFloatError(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::SqlxError(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::MigrateError(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::VarError(err) => ApplicationError::Internal(err.to_string()),
      ServerError::Dotenv(err) => ApplicationError::Internal(err.to_string()),
      ServerError::TokioTaskJoinError(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::Multipart(err) => {
        ApplicationError::BadRequest(err.to_string())
      }
      ServerError::DataError(err) => {
        ApplicationError::BadRequest(err.to_string())
      }
      ServerError::Processing(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::SerdeJson(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::Jwt(err) => ApplicationError::Internal(err.to_string()),
      ServerError::PasswordHash(err) => {
        ApplicationError::Internal(err.to_string())
      }
      ServerError::OtherError(err) => {
        ApplicationError::Internal(err.to_string())
      }
    }
  }
}
