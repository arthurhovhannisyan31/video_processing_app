use std::sync::Arc;
use std::time::Duration;

use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::core::email::EmailSender;
use crate::core::error::ApplicationError;
use crate::core::jwt::{JwtService, hash_password, verify_password};
use crate::features::auth::constants::{
  INVALID_RESET_TOKEN, PASSWORD_RESET_COOLDOWN_SEC, PASSWORD_RESET_EMAIL_SUBJECT,
  PASSWORD_RESET_TOKEN_BYTES,
};
use crate::features::auth::model::{User, UserId};
use crate::features::auth::repository::UserRepository;

#[derive(Clone)]
pub struct PasswordResetConfig {
  pub frontend_url: String,
  pub token_ttl: Duration,
}

#[derive(Clone)]
pub struct AuthService<R: UserRepository + 'static> {
  repo: R,
  jwt_service: JwtService,
  email_sender: Arc<dyn EmailSender>,
  password_reset_config: PasswordResetConfig,
}

impl<R> AuthService<R>
where
  R: UserRepository + 'static,
{
  pub fn new(
    repo: R,
    jwt_service: JwtService,
    email_sender: Arc<dyn EmailSender>,
    password_reset_config: PasswordResetConfig,
  ) -> Self {
    Self {
      repo,
      jwt_service,
      email_sender,
      password_reset_config,
    }
  }

  pub async fn get(&self, id: UserId) -> Result<User, ApplicationError> {
    self
      .repo
      .find_by_id(id.clone())
      .await?
      .ok_or_else(|| ApplicationError::NotFound(format!("user {}", id)))
  }

  pub async fn get_by_email(&self, email: &str) -> Result<User, ApplicationError> {
    match self.repo.find_by_email(&email.to_lowercase()).await {
      Ok(Some(user)) => Ok(user),
      Ok(None) => Err(ApplicationError::NotFound(format!("user {}", email))),
      Err(err) => Err(err)?,
    }
  }

  pub async fn register(
    &self,
    email: String,
    password: String,
    username: String,
  ) -> Result<User, ApplicationError> {
    let hash = hash_password(&password)?;
    let user = User::new(email.to_lowercase(), hash, username);

    self.repo.create(user).await.map_err(ApplicationError::from)
  }

  pub async fn login(
    &self,
    email: &str,
    password: &str,
    mock_password_hash: &str,
  ) -> Result<(User, String), ApplicationError> {
    let user_res = self.repo.find_by_email(&email.to_lowercase()).await?;

    let password_hash = user_res
      .as_ref()
      .map(|u| u.password_hash.as_str())
      .unwrap_or(mock_password_hash);

    // constant performance time for any user lookup result
    let password_ok = match verify_password(password, password_hash) {
      Ok(password_ok) => password_ok,
      Err(err) => return Err(ApplicationError::Internal(err.to_string())),
    };

    let user = match (user_res, password_ok) {
      (Some(user), true) => user,
      _ => return Err(ApplicationError::Unauthorized),
    };

    let token = self
      .jwt_service
      .generate_token(user.id.clone(), user.username.clone())
      .map_err(|err| ApplicationError::Internal(err.to_string()))?;

    Ok((user, token))
  }

  /// Issues a reset token and emails it to the user.
  /// Unknown emails and requests within the cooldown are silently ignored
  /// so the caller can't distinguish them.
  pub async fn request_password_reset(&self, email: &str) -> Result<(), ApplicationError> {
    let Some(user) = self.repo.find_by_email(&email.to_lowercase()).await? else {
      return Ok(());
    };

    if let Some(created_at) = self
      .repo
      .latest_reset_token_created_at(user.id.clone())
      .await?
      && Utc::now() - created_at < chrono::Duration::seconds(PASSWORD_RESET_COOLDOWN_SEC)
    {
      info!(user_id = %user.id, "password reset requested within cooldown, skipping");
      return Ok(());
    }

    let mut token_bytes = [0u8; PASSWORD_RESET_TOKEN_BYTES];
    OsRng.fill_bytes(&mut token_bytes);
    let token = URL_SAFE_NO_PAD.encode(token_bytes);

    let ttl = chrono::Duration::from_std(self.password_reset_config.token_ttl)
      .map_err(|err| ApplicationError::Internal(err.to_string()))?;
    self
      .repo
      .create_reset_token(user.id.clone(), &hash_reset_token(&token), Utc::now() + ttl)
      .await?;

    let body = format!(
      "Hi {username},\n\n\
      We received a request to reset your password. Open the link below to choose a new one:\n\n\
      {frontend_url}/reset-password?token={token}\n\n\
      The link expires in {ttl_min} minutes and can be used once.\n\
      If you didn't request a password reset, you can ignore this email.\n",
      username = user.username,
      frontend_url = self.password_reset_config.frontend_url,
      ttl_min = ttl.num_minutes(),
    );
    self
      .email_sender
      .send(&user.email, PASSWORD_RESET_EMAIL_SUBJECT, body)
      .await?;
    info!(user_id = %user.id, "password reset email sent");

    Ok(())
  }

  pub async fn reset_password(&self, token: &str, password: &str) -> Result<(), ApplicationError> {
    let password_hash = hash_password(password)?;

    let consumed = self
      .repo
      .consume_reset_token_and_set_password(&hash_reset_token(token), &password_hash)
      .await?;

    if !consumed {
      return Err(ApplicationError::BadRequest(
        INVALID_RESET_TOKEN.to_string(),
      ));
    }

    Ok(())
  }
}

fn hash_reset_token(token: &str) -> Vec<u8> {
  Sha256::digest(token.as_bytes()).to_vec()
}
