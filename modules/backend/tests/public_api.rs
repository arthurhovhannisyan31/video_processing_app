mod utils;

#[cfg(test)]
mod test_public_api {
  use axum::http::StatusCode;
  use axum_test::{TestServer, expect_json};
  use serde_json::json;
  use sqlx::PgPool;
  use uuid::{Uuid, Version};
  use video_processing_server::core::error::ServerError;
  use video_processing_server::features::auth::dto::{
    AuthRequest, AuthResponse, CreateUserRequest,
  };
  use video_processing_server::router::routes;

  use crate::utils;

  fn is_valid_v4_uuid(input: &str) -> bool {
    match Uuid::parse_str(input) {
      Ok(parsed_uuid) => parsed_uuid.get_version() == Some(Version::Random),
      Err(_) => false,
    }
  }

  #[sqlx::test]
  async fn test_health(pool: PgPool) -> Result<(), ServerError> {
    let router = utils::setup_router(pool)?;
    let server = TestServer::new(router);

    let response = server
      .get(&utils::with_base_route(routes::HEALTH))
      .expect_success()
      .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    response.assert_json(&json!({
      "status": expect_json::string(),
      "timestamp": expect_json::string(),
    }));

    Ok(())
  }

  #[sqlx::test]
  async fn test_openapi(pool: PgPool) -> Result<(), ServerError> {
    let router = utils::setup_router(pool)?;
    let server = TestServer::new(router);

    let response = server
      .get(&utils::with_base_route(routes::OPENAPI))
      .expect_success()
      .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    response.assert_json(&json!({
      "openapi": expect_json::string(),
      "info": expect_json::object(),
      "paths": expect_json::object(),
      "components": expect_json::object(),
    }));

    Ok(())
  }

  #[ignore]
  #[sqlx::test]
  async fn test_register(pool: PgPool) -> Result<(), ServerError> {
    let router = utils::setup_router(pool)?;
    let server = TestServer::new(router);

    let create_user_request = CreateUserRequest {
      email: "test@test.com".into(),
      username: "testtest".into(),
      password: "Testtest1!".into(),
    };
    let response = server
      .post(&utils::with_base_route(routes::REGISTER))
      .add_header("X-Forwarded-For", "127.0.0.1")
      .json(&json!(create_user_request))
      .expect_success()
      .await;
    let auth_response = response.json::<AuthResponse>();

    assert_eq!(response.status_code(), StatusCode::CREATED);
    assert_eq!(auth_response.user.username, create_user_request.username);
    assert_eq!(auth_response.user.email, create_user_request.email);
    assert!(is_valid_v4_uuid(&auth_response.user.user_id));

    Ok(())
  }

  #[ignore]
  #[sqlx::test(fixtures("create_user"))]
  async fn test_login(pool: PgPool) -> Result<(), ServerError> {
    let router = utils::setup_router(pool)?;
    let server = TestServer::new(router);

    let authentication_request = AuthRequest {
      email: "test@test.com".into(),
      password: "Testtest1!".into(),
    };

    let response = server
      .post(&utils::with_base_route(routes::LOGIN))
      .add_header("X-Forwarded-For", "127.0.0.1")
      .json(&json!(authentication_request))
      .expect_success()
      .await;
    let auth_response = response.json::<AuthResponse>();

    assert_eq!(response.status_code(), StatusCode::OK);
    assert_eq!(auth_response.user.username, "testtest");
    assert_eq!(auth_response.user.email, "test@test.com");
    assert!(is_valid_v4_uuid(&auth_response.user.user_id));

    Ok(())
  }
}
