use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{error, info};

use crate::core::error::DomainError;
use crate::features::auth::constants::{USERS_EMAIL_CONSTRAINT, USERS_USERNAME_CONSTRAINT};
use crate::features::auth::model::{User, UserId};

#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn create(&self, user: User) -> Result<User, DomainError>;
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError>;
  /// Replaces any previously issued reset tokens of the user with a new one
  async fn create_reset_token(
    &self,
    user_id: UserId,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
  ) -> Result<(), DomainError>;
  async fn latest_reset_token_created_at(
    &self,
    user_id: UserId,
  ) -> Result<Option<DateTime<Utc>>, DomainError>;
  /// Atomically consumes a valid reset token and updates the owner's password.
  /// Returns `false` when the token is unknown or expired.
  async fn consume_reset_token_and_set_password(
    &self,
    token_hash: &[u8],
    password_hash: &str,
  ) -> Result<bool, DomainError>;
}

#[derive(Clone)]
pub struct PostgresUserRepository {
  pool: PgPool,
}

impl PostgresUserRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
  async fn create(&self, user: User) -> Result<User, DomainError> {
    let row = sqlx::query_as!(
      User,
      r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING users.id as "id: UserId", users.username, users.email, users.password_hash, users.created_at, users.password_changed_at
      "#,
      user.username,
      user.email,
      user.password_hash,
    )
      .fetch_one(&self.pool)
      .await
      .map_err(|e| {
        error!("Failed to create user: {}", e);
        if e
          .as_database_error()
          .and_then(|db| db.constraint())
          .map(|c| {
            c.contains(USERS_USERNAME_CONSTRAINT) || c.contains(USERS_EMAIL_CONSTRAINT)
          })
          == Some(true)
        {
          DomainError::UserAlreadyExists
        } else {
          DomainError::Internal(format!("database error: {}", e))
        }
      })?;

    info!(user_id = %user.id, email = %user.email, "user created");
    Ok(row)
  }
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
    let row = sqlx::query_as!(
      User,
      r#"
        SELECT users.id as "id: UserId", users.username, users.email, users.password_hash, users.created_at, users.password_changed_at
        FROM users
        WHERE users.email = $1
      "#,
      email
    ).fetch_optional(&self.pool)
      .await
      .inspect_err(|err| {
        error!("Failed to find user by email {}: {}", email, err)
        })
      ?;

    Ok(row)
  }
  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
    let row = sqlx::query_as!(
      User,
      r#"
        SELECT users.id as "id: UserId", users.username, users.email, users.password_hash, users.created_at, users.password_changed_at
        FROM users
        WHERE users.id = $1
      "#,
      id as _
    ).fetch_optional(&self.pool)
      .await
      .inspect_err(
        |err| {
          error!("Failed to find user by id {}: {}", id,err);
        }
      )?;

    Ok(row)
  }
  async fn create_reset_token(
    &self,
    user_id: UserId,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
  ) -> Result<(), DomainError> {
    let mut tx = self.pool.begin().await?;

    sqlx::query_scalar!(
      "DELETE FROM password_reset_tokens WHERE user_id = $1 RETURNING id",
      user_id.clone() as _
    )
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query_scalar!(
      r#"
        INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id
      "#,
      user_id.clone() as _,
      token_hash,
      expires_at,
    )
    .fetch_one(&mut *tx)
    .await
    .inspect_err(|err| error!("Failed to create reset token for user {}: {}", user_id, err))?;

    tx.commit().await?;

    Ok(())
  }
  async fn latest_reset_token_created_at(
    &self,
    user_id: UserId,
  ) -> Result<Option<DateTime<Utc>>, DomainError> {
    let created_at = sqlx::query_scalar!(
      "SELECT MAX(created_at) FROM password_reset_tokens WHERE user_id = $1",
      user_id as _
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(created_at)
  }
  async fn consume_reset_token_and_set_password(
    &self,
    token_hash: &[u8],
    password_hash: &str,
  ) -> Result<bool, DomainError> {
    let mut tx = self.pool.begin().await?;

    // Get user id out of many reset requests
    let user_id = sqlx::query_scalar!(
      r#"
        DELETE FROM password_reset_tokens
        WHERE token_hash = $1 AND expires_at > NOW()
        RETURNING user_id as "user_id: UserId"
      "#,
      token_hash
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(user_id) = user_id else {
      return Ok(false);
    };

    sqlx::query_scalar!(
      r#"
        UPDATE users
        SET password_hash = $1, password_changed_at = NOW()
        WHERE id = $2
        RETURNING id
      "#,
      password_hash,
      user_id.clone() as _
    )
    .fetch_one(&mut *tx)
    .await?;

    // Cleanup unused reset tokens
    sqlx::query_scalar!(
      "DELETE FROM password_reset_tokens WHERE user_id = $1 RETURNING id",
      user_id.clone() as _
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    info!(user_id = %user_id, "user password reset");

    Ok(true)
  }
}
