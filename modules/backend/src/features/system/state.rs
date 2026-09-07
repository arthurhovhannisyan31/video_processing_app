use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;

use crate::core::app_state::AppState;

pub struct SystemState {
  pub db_pool: PgPool,
}

impl FromRef<AppState> for Arc<SystemState> {
  fn from_ref(app_state: &AppState) -> Self {
    app_state.system_state.clone()
  }
}
