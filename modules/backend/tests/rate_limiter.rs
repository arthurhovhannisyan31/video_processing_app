mod constants;
mod utils;

#[cfg(test)]
mod test_rate_limiter {
  use std::env;
  use std::time::Duration;

  use axum::http::StatusCode;
  use axum_test::TestServer;
  use sqlx::PgPool;
  use video_processing_server::core::error::ApplicationError;
  use video_processing_server::core::extractors::X_USER_ID_HEADER;
  use video_processing_server::router::routes;

  use crate::constants::MOCK_USER_ID;
  use crate::utils::{setup_router, with_base_route};

  #[sqlx::test()]
  async fn test_video_rate_limiter_size(pool: PgPool) -> Result<(), ApplicationError> {
    let rate_limit_size: u64 = 10;
    unsafe {
      env::set_var("IS_PRODUCTION", "true");
      env::set_var("BACKEND_VIDEO_RATE_LIMIT_SIZE", rate_limit_size.to_string());
    }

    let router = setup_router(pool)?;
    let server = TestServer::new(router);

    for _ in 0..rate_limit_size {
      let response = server
        .post(&with_base_route(routes::VIDEO_INSPECT))
        .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
        .add_header("X-Forwarded-For", "0.0.0.0")
        .await;
      assert_ne!(response.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }

    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .add_header("X-Forwarded-For", "0.0.0.0")
      .await;
    assert_eq!(response.status_code(), StatusCode::TOO_MANY_REQUESTS);

    Ok(())
  }

  #[sqlx::test()]
  async fn test_video_rate_limiter_period(pool: PgPool) -> Result<(), ApplicationError> {
    let rate_limit_size: u64 = 10;
    let rate_limit_period: u64 = 1;
    unsafe {
      env::set_var("IS_PRODUCTION", "true");
      env::set_var("BACKEND_VIDEO_RATE_LIMIT_SIZE", rate_limit_size.to_string());
      env::set_var(
        "BACKEND_VIDEO_RATE_LIMIT_PERIOD_SEC",
        rate_limit_period.to_string(),
      );
    }

    let router = setup_router(pool)?;
    let server = TestServer::new(router);

    // Spend all available requests quota
    for _ in 0..rate_limit_size {
      let response = server
        .post(&with_base_route(routes::VIDEO_INSPECT))
        .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
        .add_header("X-Forwarded-For", "0.0.0.0")
        .await;
      assert_ne!(response.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }

    // Wait not long enough and fail
    tokio::time::sleep(Duration::from_millis(900)).await;

    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .add_header("X-Forwarded-For", "0.0.0.0")
      .await;
    assert_eq!(response.status_code(), StatusCode::TOO_MANY_REQUESTS);

    // Wait up to 1000 ms and succeed
    tokio::time::sleep(Duration::from_millis(100)).await;

    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .add_header("X-Forwarded-For", "0.0.0.0")
      .await;
    assert_ne!(response.status_code(), StatusCode::TOO_MANY_REQUESTS);

    Ok(())
  }
}
