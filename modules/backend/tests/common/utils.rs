use std::sync::Arc;

use axum::Router;
use axum_test::TestServer;
use cookie::CookieJar;
use parking_lot::Mutex;
use serde_json::json;
use sqlx::PgPool;
use video_processing_server::core::app_config::AppConfig;
use video_processing_server::core::app_state::AppState;
use video_processing_server::core::email::MockEmailSender;
use video_processing_server::core::error::ServerError;
use video_processing_server::core::jwt::JwtService;
use video_processing_server::core::session::build_session_store;
use video_processing_server::features::auth::dto::{AuthRequest, AuthResponse};
use video_processing_server::features::auth::repository::PostgresUserRepository;
use video_processing_server::features::auth::service::{AuthService, PasswordResetConfig};
use video_processing_server::features::auth::state::AuthState;
use video_processing_server::features::system::state::SystemState;
use video_processing_server::features::video::service::VideoService;
use video_processing_server::features::video::state::VideoState;
use video_processing_server::router::{build_router, routes};

use super::constants::MOCK_SESSION_ID;

#[cfg(test)]
#[allow(dead_code)]
pub async fn setup_router(pool: PgPool) -> Result<Router, ServerError> {
  let (router, _) = setup_router_with_email(pool).await?;

  Ok(router)
}

/// Same as `setup_router`, also returns the email sender capturing sent emails
#[cfg(test)]
#[allow(dead_code)]
pub async fn setup_router_with_email(
  pool: PgPool,
) -> Result<(Router, MockEmailSender), ServerError> {
  let app_config = AppConfig::from_env()?;
  let session_store = build_session_store(pool.clone(), app_config.is_production).await?;

  let email_sender = MockEmailSender::default();
  let jwt_service = JwtService::new(app_config.jwt_secret.clone());
  let users_repo = PostgresUserRepository::new(pool.clone());
  let auth_service = AuthService::new(
    users_repo,
    jwt_service.clone(),
    Arc::new(email_sender.clone()),
    PasswordResetConfig {
      frontend_url: app_config.frontend_url.clone(),
      token_ttl: app_config.password_reset_ttl,
    },
  );
  let app_state = AppState {
    auth_state: Arc::new(AuthState {
      auth_service,
      jwt_service,
    }),
    app_config: Arc::new(app_config),
    video_state: Arc::from(VideoState {
      video_service: VideoService::default(),
    }),
    system_state: Arc::new(SystemState {
      db_pool: pool,
      rate_limiters: Arc::new(Mutex::new(vec![])),
    }),
  };

  Ok((build_router(app_state, session_store)?, email_sender))
}

#[cfg(test)]
#[allow(dead_code)]
pub fn with_base_route(path: &str) -> String {
  format!("/api/{}", path.strip_prefix("/").unwrap())
}

#[cfg(test)]
#[allow(dead_code)]
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

#[cfg(test)]
#[allow(dead_code)]
pub fn get_session_cookie_jar() -> CookieJar {
  let mut jar = CookieJar::new();

  jar.add(("session", MOCK_SESSION_ID));

  jar
}
