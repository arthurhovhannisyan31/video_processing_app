use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::core::error::ServerError;
use crate::features::auth::model::UserId;

pub const TOKEN_EXPIRATION_HOURS: i64 = 24;

#[derive(Clone)]
pub struct JwtService {
  secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub user_id: UserId,
  pub username: String,
  pub exp: usize,
}

impl JwtService {
  pub fn new(secret: String) -> Self {
    Self { secret }
  }

  pub fn generate_token(&self, user_id: UserId, username: String) -> Result<String, ServerError> {
    let claims = Claims {
      user_id,
      username,
      exp: chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(TOKEN_EXPIRATION_HOURS))
        .expect("token expiration duration does not overflow")
        .timestamp() as usize,
    };
    Ok(encode(
      &Header::default(),
      &claims,
      &EncodingKey::from_secret(self.secret.as_bytes()),
    )?)
  }

  pub fn verify_token(&self, token: &str) -> Result<Claims, ServerError> {
    let data = decode::<Claims>(
      token,
      &DecodingKey::from_secret(self.secret.as_bytes()),
      &Validation::default(),
    )?;
    Ok(data.claims)
  }
}

pub fn hash_password(password: &str) -> Result<String, ServerError> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let hash = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();
  Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
  let parsed = PasswordHash::new(hash)?;
  let argon2 = Argon2::default();
  Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}
