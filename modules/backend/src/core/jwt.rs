use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::{Argon2, PasswordHash};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::core::error::ServerError;
use crate::features::auth::model::UserId;

pub const TOKEN_EXPIRATION_HOURS: i64 = 24;

#[derive(Clone)]
pub struct JwtService {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub user_id: UserId,
  pub username: String,
  pub exp: usize,
  pub iat: usize,
}

impl JwtService {
  pub fn new(secret: String) -> Self {
    Self {
      encoding_key: EncodingKey::from_secret(secret.as_bytes()),
      decoding_key: DecodingKey::from_secret(secret.as_bytes()),
      validation: Validation::default(),
    }
  }

  pub fn generate_token(&self, user_id: UserId, username: String) -> Result<String, ServerError> {
    let now = chrono::Utc::now();
    let claims = Claims {
      user_id,
      username,
      iat: now.timestamp() as usize,
      exp: now
        .checked_add_signed(chrono::Duration::hours(TOKEN_EXPIRATION_HOURS))
        .expect("token expiration duration does not overflow")
        .timestamp() as usize,
    };
    Ok(encode(&Header::default(), &claims, &self.encoding_key)?)
  }

  pub fn verify_token(&self, token: &str) -> Result<Claims, ServerError> {
    let data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
    Ok(data.claims)
  }
}

pub fn hash_password(password: &str) -> Result<String, ServerError> {
  let argon2 = Argon2::default();
  let hash = argon2.hash_password(password.as_bytes())?.to_string();
  Ok(hash)
}

/// Runs `hash_password` on the blocking pool, Argon2 takes tens of ms of CPU
pub async fn hash_password_blocking(password: String) -> Result<String, ServerError> {
  tokio::task::spawn_blocking(move || hash_password(&password))
    .await
    .map_err(|err| ServerError::Processing(format!("Password hashing task failed: {err}")))?
}

/// Runs `verify_password` on the blocking pool, Argon2 takes tens of ms of CPU
pub async fn verify_password_blocking(password: String, hash: String) -> Result<bool, ServerError> {
  tokio::task::spawn_blocking(move || verify_password(&password, &hash))
    .await
    .map_err(|err| ServerError::Processing(format!("Password verify task failed: {err}")))?
    .map_err(ServerError::from)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
  let parsed = PasswordHash::new(hash)?;
  let argon2 = Argon2::default();
  Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}
