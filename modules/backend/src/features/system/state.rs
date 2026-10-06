use std::net::IpAddr;
use std::sync::Arc;

use axum::extract::FromRef;
use governor::middleware::NoOpMiddleware;
use parking_lot::Mutex;
use sqlx::PgPool;
use tower_governor::governor::SharedRateLimiter;

use crate::core::app_state::AppState;

pub type RateLimiters = Vec<SharedRateLimiter<IpAddr, NoOpMiddleware>>;

pub struct SystemState {
  pub db_pool: PgPool,
  pub rate_limiters: Arc<Mutex<RateLimiters>>,
}

impl FromRef<AppState> for Arc<SystemState> {
  fn from_ref(app_state: &AppState) -> Self {
    app_state.system_state.clone()
  }
}
