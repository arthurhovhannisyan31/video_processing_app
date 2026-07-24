use sqlx::Error;
use std::{io, net::AddrParseError, num::ParseIntError};
use thiserror::Error;

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
