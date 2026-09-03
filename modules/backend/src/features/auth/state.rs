use std::sync::Arc;

use axum::extract::FromRef;

use crate::core::app_state::AppState;
use crate::core::jwt::JwtService;
use crate::features::auth::repository::PostgresUserRepository;
use crate::features::auth::service::AuthService;

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
