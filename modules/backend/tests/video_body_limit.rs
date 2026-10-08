mod constants;
mod utils;

/// Separate test binary: `BACKEND_VIDEO_MAX_BODY_SIZE` is set via env var for the whole process
#[cfg(test)]
mod test_video_body_limit {
  use std::env;

  use axum::http::{StatusCode, header};
  use axum_test::TestServer;
  use axum_test::multipart::{MultipartForm, Part};
  use sqlx::PgPool;
  use video_processing_server::core::error::{ErrorBody, ServerError};
  use video_processing_server::core::extractors::X_USER_ID_HEADER;
  use video_processing_server::router::routes;

  use crate::constants::MOCK_USER_ID;
  use crate::utils::{setup_router, with_base_route};

  const MAX_BODY_SIZE: usize = 1024 * 1024;

  fn setup_server(pool: PgPool) -> Result<TestServer, ServerError> {
    unsafe {
      env::set_var("BACKEND_VIDEO_MAX_BODY_SIZE", MAX_BODY_SIZE.to_string());
    }
    let router = setup_router(pool)?;

    Ok(TestServer::new(router))
  }

  fn video_form(file_bytes: &[u8], file_name: &str) -> MultipartForm {
    let part_bytes = Part::bytes(file_bytes.to_vec())
      .file_name(file_name.to_string())
      .mime_type("video/mp4");
    MultipartForm::new().add_part("video", part_bytes)
  }

  async fn assert_body_too_large(
    server: TestServer,
    route: &str,
    form: MultipartForm,
  ) -> Result<(), ServerError> {
    let response = server
      .post(&with_base_route(route))
      .multipart(form)
      .add_header(header::AUTHORIZATION, "temporary-disabled")
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let error_body = serde_json::from_str::<ErrorBody>(&response.text())?;
    assert!(
      error_body.message.starts_with("Multipart form error:"),
      "unexpected message: {}",
      error_body.message
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_inspect_body_over_limit(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool)?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/2_7_mb.mp4");
    assert!(file_bytes.len() > MAX_BODY_SIZE);
    let form = video_form(file_bytes, "2_7_mb.mp4");

    assert_body_too_large(server, routes::VIDEO_INSPECT, form).await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_process_body_over_limit(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool)?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/2_7_mb.mp4");
    let form = video_form(file_bytes, "2_7_mb.mp4").add_text("operation", "compress");

    assert_body_too_large(server, routes::VIDEO_JOBS, form).await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_inspect_body_under_limit(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool)?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/sample_av.mp4");
    assert!(file_bytes.len() < MAX_BODY_SIZE);
    let form = video_form(file_bytes, "sample_av.mp4");

    server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .multipart(form)
      .add_header(header::AUTHORIZATION, "temporary-disabled")
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .await
      .assert_status_ok();

    Ok(())
  }
}
