use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
  #[error("Access is forbidden")]
  Forbidden,
  #[error("Invalid credentials")]
  InvalidCredentials,
  #[error("internal error: {0}")]
  Internal(String),
  #[error("Post not found: {0}")]
  PostNotFound(u64),
  #[error("User already exists")]
  UserAlreadyExists,
  #[error("User not found: {0}")]
  UserNotFound(i64),
  #[error("Validation failed: {0}")]
  Validation(String),
}
