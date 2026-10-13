use axum_session::{SessionConfig, SessionStore};
use axum_session_sqlx::SessionPgPool;
use sqlx::PgPool;

use crate::core::error::{ServerError, StartupError};
use crate::features::auth::constants::SESSION_COOKIE_PATH;

pub async fn build_session_store(
  pool: PgPool,
  is_production: bool,
) -> Result<SessionStore<SessionPgPool>, ServerError> {
  let session_config = SessionConfig::default()
    .with_cookie_path(SESSION_COOKIE_PATH)
    .with_secure(is_production)
    .with_table_name("sessions_table");
  let session_store = SessionStore::<SessionPgPool>::new(Some(pool.clone().into()), session_config)
    .await
    .map_err(StartupError::SessionError)?;

  Ok(session_store)
}
