use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(pub Uuid);

impl Display for UserId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
  pub id: UserId,
  pub username: String,
  pub email: String,
  pub password_hash: String,
  pub created_at: DateTime<Utc>,
}

impl User {
  pub fn new(email: String, password_hash: String, username: String) -> Self {
    Self {
      id: UserId(Uuid::new_v4()),
      username,
      email,
      password_hash,
      created_at: Utc::now(),
    }
  }
}
