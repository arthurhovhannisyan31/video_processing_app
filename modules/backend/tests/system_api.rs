mod common;
use common::utils;

#[cfg(test)]
mod test_system_api {
  use axum::http::StatusCode;
  use axum_test::{TestServer, expect_json};
  use serde_json::json;
  use sqlx::PgPool;
  use video_processing_server::core::error::ServerError;
  use video_processing_server::router::routes;

  use crate::utils;

  #[sqlx::test]
  async fn test_health(pool: PgPool) -> Result<(), ServerError> {
    let router = utils::setup_router(pool).await?;
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
    let router = utils::setup_router(pool).await?;
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
}
