use crate::data::constants::db_constraints;
use crate::domain::{
  error::DomainError,
  user::{User, UserId},
};

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{error, info};

#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn create(&self, user: User) -> Result<User, DomainError>;
  async fn find_by_email(
    &self,
    email: &str,
  ) -> Result<Option<User>, DomainError>;
  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError>;
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
        RETURNING users.id as "id: UserId", users.username, users.email, users.password_hash, users.created_at
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
            c.contains(db_constraints::USERS_USERNAME) || c.contains(db_constraints::USERS_EMAIL)
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
  async fn find_by_email(
    &self,
    email: &str,
  ) -> Result<Option<User>, DomainError> {
    let row = sqlx::query_as!(
      User,
      r#"
        SELECT users.id as "id: UserId", users.username, users.email, users.password_hash, users.created_at
        FROM users
        WHERE users.email = $1
      "#,
      email
    ).fetch_optional(&self.pool)
      .await
      .map_err(|e| {
        error!("Failed to find user by email {}: {}", email, e);
        DomainError::Internal(format!("database error: {}", e))
      })?;

    Ok(row)
  }
  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
    let row = sqlx::query_as!(
      User,
      r#"
        SELECT users.id as "id: UserId", users.username, users.email, users.password_hash, users.created_at
        FROM users
        WHERE users.id = $1
      "#,
      id.0
    ).fetch_optional(&self.pool)
      .await
      .map_err(|e| {
        error!("Failed to find user by id {}: {}", id, e);
        DomainError::Internal(format!("database error: {}", e))
      })?;

    Ok(row)
  }
}
