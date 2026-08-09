use std::sync::Arc;

use axum::Router;
use axum_test::TestServer;
use backend::core::app_config::AppConfig;
use backend::core::app_state::{AppState, AuthState};
use backend::core::error::ServerError;
use backend::core::jwt::JwtService;
use backend::features::auth::dto::{AuthRequest, AuthResponse};
use backend::features::auth::repository::PostgresUserRepository;
use backend::features::auth::service::AuthService;
use backend::router::{build_router, routes};
use serde_json::json;
use sqlx::PgPool;

#[cfg(test)]
pub fn setup_router(pool: PgPool) -> Result<Router, ServerError> {
  let app_config = AppConfig::from_env()?;
  let jwt_service = JwtService::new(app_config.jwt_secret.clone());
  let users_repo = PostgresUserRepository::new(pool.clone());
  let auth_service = AuthService::new(users_repo, jwt_service.clone());
  let app_state = AppState {
    auth_state: Arc::new(AuthState {
      auth_service,
      jwt_service,
    }),
    app_config: Arc::new(app_config),
  };

  build_router(app_state)
}

#[cfg(test)]
pub fn with_base_route(path: &str) -> String {
  format!("/api/{}", path.strip_prefix("/").unwrap())
}

#[cfg(test)]
pub async fn get_authorization_token(server: &TestServer) -> String {
  let authentication_request = AuthRequest {
    email: "test@test.com".into(),
    password: "Testtest1!".into(),
  };

  let response = server
    .post(&with_base_route(routes::LOGIN))
    .json(&json!(authentication_request))
    .expect_success()
    .await;
  let auth_response = response.json::<AuthResponse>();

  format!("Bearer {}", auth_response.token)
}
