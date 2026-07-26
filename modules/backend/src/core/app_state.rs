use crate::core::app_config::AppConfig;
use crate::core::jwt::JwtService;
use crate::features::auth::repository::PostgresUserRepository;
use crate::features::auth::service::AuthService;

use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
  pub auth_state: Arc<AuthState>,
  pub app_config: Arc<AppConfig>,
}

#[derive(Clone)]
pub struct AuthState {
  pub auth_service: AuthService<PostgresUserRepository>,
  pub jwt_service: JwtService,
}

impl FromRef<AppState> for Arc<AuthState> {
  fn from_ref(app_state: &AppState) -> Self {
    app_state.auth_state.clone()
  }
}

impl FromRef<AppState> for Arc<AppConfig> {
  fn from_ref(app_state: &AppState) -> Self {
    app_state.app_config.clone()
  }
}
