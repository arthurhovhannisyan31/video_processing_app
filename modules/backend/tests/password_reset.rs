mod common;
use common::utils;

#[cfg(test)]
mod test_password_reset {
  use std::sync::Arc;
  use std::time::Duration;

  use axum::http::StatusCode;
  use axum_test::TestServer;
  use serde_json::json;
  use sqlx::PgPool;
  use video_processing_server::core::app_config::AppConfig;
  use video_processing_server::core::email::{MockEmailSender, SentEmail};
  use video_processing_server::core::error::ServerError;
  use video_processing_server::core::jwt::JwtService;
  use video_processing_server::features::auth::dto::AuthResponse;
  use video_processing_server::features::auth::middleware::authenticate_user;
  use video_processing_server::features::auth::repository::PostgresUserRepository;
  use video_processing_server::features::auth::service::{AuthService, PasswordResetConfig};
  use video_processing_server::router::routes;

  use crate::utils;

  const EMAIL: &str = "test@test.com";
  const OLD_PASSWORD: &str = "Testtest1!";
  const NEW_PASSWORD: &str = "Newpassword2@";
  const EMAIL_WAIT_TIMEOUT: Duration = Duration::from_secs(5);
  const NO_EMAIL_WAIT: Duration = Duration::from_millis(300);

  /// Reset emails are sent from a spawned task, so poll until the expected count is reached
  async fn wait_for_emails(email_sender: &MockEmailSender, count: usize) -> Vec<SentEmail> {
    let started = tokio::time::Instant::now();
    loop {
      let sent = email_sender.sent.lock().clone();
      if sent.len() >= count || started.elapsed() > EMAIL_WAIT_TIMEOUT {
        return sent;
      }
      tokio::time::sleep(Duration::from_millis(20)).await;
    }
  }

  fn extract_token(email: &SentEmail) -> String {
    email
      .body
      .split("token=")
      .nth(1)
      .and_then(|rest| rest.split_whitespace().next())
      .expect("reset email contains a token link")
      .to_string()
  }

  async fn request_reset(server: &TestServer, email: &str) {
    let response = server
      .post(&utils::with_base_route(routes::PASSWORD_FORGOT))
      .add_header("X-Forwarded-For", "127.0.0.1")
      .json(&json!({ "email": email }))
      .await;
    assert_eq!(response.status_code(), StatusCode::ACCEPTED);
  }

  async fn reset_password(server: &TestServer, token: &str, password: &str) -> StatusCode {
    server
      .post(&utils::with_base_route(routes::PASSWORD_RESET))
      .add_header("X-Forwarded-For", "127.0.0.1")
      .json(&json!({ "token": token, "password": password }))
      .await
      .status_code()
  }

  async fn login(server: &TestServer, password: &str) -> (StatusCode, Option<AuthResponse>) {
    let response = server
      .post(&utils::with_base_route(routes::LOGIN))
      .add_header("X-Forwarded-For", "127.0.0.1")
      .json(&json!({ "email": EMAIL, "password": password }))
      .await;
    let status = response.status_code();
    let body = status.is_success().then(|| response.json::<AuthResponse>());

    (status, body)
  }

  async fn password_hash(pool: &PgPool) -> Result<String, ServerError> {
    Ok(
      sqlx::query_scalar!("SELECT password_hash FROM users WHERE email = $1", EMAIL)
        .fetch_one(pool)
        .await?,
    )
  }

  async fn request_and_get_token(server: &TestServer, email_sender: &MockEmailSender) -> String {
    request_reset(server, EMAIL).await;
    let sent = wait_for_emails(email_sender, 1).await;
    assert_eq!(sent.len(), 1);

    extract_token(&sent[0])
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_forgot_known_email_sends_link(pool: PgPool) -> Result<(), ServerError> {
    let (router, email_sender) = utils::setup_router_with_email(pool).await?;
    let server = TestServer::new(router);

    // Email lookup is case-insensitive
    request_reset(&server, "TEST@test.com").await;
    let sent = wait_for_emails(&email_sender, 1).await;

    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, EMAIL);
    assert!(sent[0].body.contains("/reset-password?token="));

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_forgot_unknown_email_is_silent(pool: PgPool) -> Result<(), ServerError> {
    let (router, email_sender) = utils::setup_router_with_email(pool).await?;
    let server = TestServer::new(router);

    request_reset(&server, "unknown@test.com").await;
    tokio::time::sleep(NO_EMAIL_WAIT).await;

    assert!(email_sender.sent.lock().is_empty());

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_forgot_invalid_email_is_rejected(pool: PgPool) -> Result<(), ServerError> {
    let (router, _) = utils::setup_router_with_email(pool).await?;
    let server = TestServer::new(router);

    let response = server
      .post(&utils::with_base_route(routes::PASSWORD_FORGOT))
      .add_header("X-Forwarded-For", "127.0.0.1")
      .json(&json!({ "email": "not-an-email" }))
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_forgot_cooldown(pool: PgPool) -> Result<(), ServerError> {
    let (router, email_sender) = utils::setup_router_with_email(pool).await?;
    let server = TestServer::new(router);

    request_and_get_token(&server, &email_sender).await;
    request_reset(&server, EMAIL).await;
    tokio::time::sleep(NO_EMAIL_WAIT).await;

    assert_eq!(email_sender.sent.lock().len(), 1);

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_new_request_voids_previous_token(pool: PgPool) -> Result<(), ServerError> {
    let (router, email_sender) = utils::setup_router_with_email(pool.clone()).await?;
    let server = TestServer::new(router);

    let first_token = request_and_get_token(&server, &email_sender).await;
    // Skip cooldown
    sqlx::query_scalar!(
      "UPDATE password_reset_tokens SET created_at = NOW() - INTERVAL '1 hour' RETURNING id"
    )
    .fetch_all(&pool)
    .await?;
    request_reset(&server, EMAIL).await;
    let sent = wait_for_emails(&email_sender, 2).await;
    assert_eq!(sent.len(), 2);
    let second_token = extract_token(&sent[1]);

    assert_eq!(
      reset_password(&server, &first_token, NEW_PASSWORD).await,
      StatusCode::BAD_REQUEST
    );
    assert_eq!(
      reset_password(&server, &second_token, NEW_PASSWORD).await,
      StatusCode::NO_CONTENT
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_reset_password_flow(pool: PgPool) -> Result<(), ServerError> {
    let (router, email_sender) = utils::setup_router_with_email(pool).await?;
    let server = TestServer::new(router);

    let token = request_and_get_token(&server, &email_sender).await;

    assert_eq!(
      reset_password(&server, &token, NEW_PASSWORD).await,
      StatusCode::NO_CONTENT
    );
    assert_eq!(
      login(&server, OLD_PASSWORD).await.0,
      StatusCode::UNAUTHORIZED
    );
    assert_eq!(login(&server, NEW_PASSWORD).await.0, StatusCode::OK);

    // Token is single use
    assert_eq!(
      reset_password(&server, &token, "Anotherpass3#").await,
      StatusCode::BAD_REQUEST
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_reset_with_expired_token(pool: PgPool) -> Result<(), ServerError> {
    let (router, email_sender) = utils::setup_router_with_email(pool.clone()).await?;
    let server = TestServer::new(router);

    let token = request_and_get_token(&server, &email_sender).await;
    sqlx::query_scalar!(
      "UPDATE password_reset_tokens SET expires_at = NOW() - INTERVAL '1 second' RETURNING id"
    )
    .fetch_all(&pool)
    .await?;

    let hash_before = password_hash(&pool).await?;

    assert_eq!(
      reset_password(&server, &token, NEW_PASSWORD).await,
      StatusCode::BAD_REQUEST
    );
    assert_eq!(password_hash(&pool).await?, hash_before);

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_reset_with_unknown_token(pool: PgPool) -> Result<(), ServerError> {
    let (router, _) = utils::setup_router_with_email(pool).await?;
    let server = TestServer::new(router);

    assert_eq!(
      reset_password(&server, "unknown-token", NEW_PASSWORD).await,
      StatusCode::BAD_REQUEST
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_reset_with_weak_password_keeps_token(pool: PgPool) -> Result<(), ServerError> {
    let (router, email_sender) = utils::setup_router_with_email(pool).await?;
    let server = TestServer::new(router);

    let token = request_and_get_token(&server, &email_sender).await;

    assert_eq!(
      reset_password(&server, &token, "weakpassword").await,
      StatusCode::BAD_REQUEST
    );
    // Validation failure doesn't consume the token
    assert_eq!(
      reset_password(&server, &token, NEW_PASSWORD).await,
      StatusCode::NO_CONTENT
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_reset_revokes_issued_jwt(pool: PgPool) -> Result<(), ServerError> {
    let (router, email_sender) = utils::setup_router_with_email(pool.clone()).await?;
    let server = TestServer::new(router);

    let app_config = AppConfig::from_env()?;
    let jwt_service = JwtService::new(app_config.jwt_secret.clone());
    let auth_service = AuthService::new(
      PostgresUserRepository::new(pool),
      jwt_service.clone(),
      Arc::new(MockEmailSender::default()),
      PasswordResetConfig {
        frontend_url: app_config.frontend_url.clone(),
        token_ttl: app_config.password_reset_ttl,
      },
    );

    let user = auth_service.get_by_email(EMAIL).await.expect("user exists");
    let old_token = jwt_service.generate_token(user.id, user.username)?;
    assert!(
      authenticate_user(&old_token, &jwt_service, &auth_service)
        .await
        .is_some()
    );

    // JWT iat has a seconds precision
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let reset_token = request_and_get_token(&server, &email_sender).await;
    assert_eq!(
      reset_password(&server, &reset_token, NEW_PASSWORD).await,
      StatusCode::NO_CONTENT
    );

    assert!(
      authenticate_user(&old_token, &jwt_service, &auth_service)
        .await
        .is_none()
    );

    let (_, new_auth) = login(&server, NEW_PASSWORD).await;
    let new_token = new_auth.expect("login succeeds").token;
    assert!(
      authenticate_user(&new_token, &jwt_service, &auth_service)
        .await
        .is_some()
    );

    Ok(())
  }
}
