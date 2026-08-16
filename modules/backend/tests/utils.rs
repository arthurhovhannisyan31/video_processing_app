use std::sync::Arc;

use axum::Router;
use axum_test::TestServer;
use serde_json::json;
use sqlx::PgPool;
use video_processing_server::core::app_config::AppConfig;
use video_processing_server::core::app_state::{AppState, AuthState};
use video_processing_server::core::error::ServerError;
use video_processing_server::core::jwt::JwtService;
use video_processing_server::features::auth::dto::{AuthRequest, AuthResponse};
use video_processing_server::features::auth::repository::PostgresUserRepository;
use video_processing_server::features::auth::service::AuthService;
use video_processing_server::router::{build_router, routes};

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
    .add_header("X-Forwarded-For", "127.0.0.1")
    .json(&json!(authentication_request))
    .expect_success()
    .await;
  let auth_response = response.json::<AuthResponse>();

  format!("Bearer {}", auth_response.token)
}
