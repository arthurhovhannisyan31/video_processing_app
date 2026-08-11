mod utils;

#[cfg(test)]
mod test_video_process_api {
  use axum::http::{StatusCode, header};
  use axum_test::multipart::{MultipartForm, Part};
  use axum_test::{TestServer, expect_json};
  use backend::core::error::ServerError;
  use backend::router::routes;
  use serde_json::json;
  use sqlx::PgPool;

  use crate::utils::{get_authorization_token, setup_router, with_base_route};

  /// Important
  ///
  /// fs::metadata read files relative to current working directory of running process
  ///
  /// include_bytes! reads files relative source file at compile time
  async fn assert_success_response(
    server: TestServer,
    form: MultipartForm,
    token: String,
  ) -> Result<(), ServerError> {
    let response = server
      .post(&with_base_route(routes::VIDEO_JOBS))
      .multipart(form)
      .add_header(header::AUTHORIZATION, token)
      .expect_success()
      .await;

    response.assert_status_ok();
    response.assert_header("content-type", "video/mp4");
    let content_disposition = response.header("content-disposition");
    assert!(
      content_disposition
        .to_str()
        .unwrap()
        .contains("attachment; filename=")
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_wrong_file_format(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "audio_only.m4a";
    let bearer_token = get_authorization_token(&server).await;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/audio_only.m4a");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("audio/x-m4a");
    let form = MultipartForm::new()
      .add_part("audio", part_bytes)
      .add_text("operation", "compress");

    let response = server
      .post(&with_base_route(routes::VIDEO_JOBS))
      .multipart(form)
      .add_header(header::AUTHORIZATION, bearer_token)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    response.assert_json(&json!({
      "message": expect_json::string(),
    }));

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_broken_video_file(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "broken_truncated.mp4";
    let bearer_token = get_authorization_token(&server).await;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/broken_truncated.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("audio/x-m4a");
    let form = MultipartForm::new()
      .add_part("audio", part_bytes)
      .add_text("operation", "compress");

    let response = server
      .post(&with_base_route(routes::VIDEO_JOBS))
      .multipart(form)
      .add_header(header::AUTHORIZATION, bearer_token)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    response.assert_json(&json!({
      "message": expect_json::string(),
    }));

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_missing_video_field(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let bearer_token = get_authorization_token(&server).await;
    let form = MultipartForm::new().add_text("operation", "compress");

    let response = server
      .post(&with_base_route(routes::VIDEO_JOBS))
      .multipart(form)
      .add_header(header::AUTHORIZATION, bearer_token)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    response.assert_json(&json!({
      "message": expect_json::string(),
    }));

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_correct_video_file_1(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "dual_audio_tracks.mp4";
    let bearer_token = get_authorization_token(&server).await;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/dual_audio_tracks.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new()
      .add_part("video", part_bytes)
      .add_text("operation", "compress");

    assert_success_response(server, form, bearer_token).await?;

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_correct_video_file_2(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "sample_av.mp4";
    let bearer_token = get_authorization_token(&server).await;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/sample_av.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new()
      .add_part("video", part_bytes)
      .add_text("operation", "compress");

    assert_success_response(server, form, bearer_token).await?;

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_correct_video_file_3(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "vertical_no_audio.mp4";
    let bearer_token = get_authorization_token(&server).await;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/vertical_no_audio.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new()
      .add_part("video", part_bytes)
      .add_text("operation", "compress");

    assert_success_response(server, form, bearer_token).await?;

    Ok(())
  }
}
