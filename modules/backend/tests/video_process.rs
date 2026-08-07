mod utils;

#[cfg(test)]
mod test_video_process_api {
  use crate::utils::{get_authorization_token, setup_router, with_base_route};
  use axum::http::{StatusCode, header};
  use axum_test::multipart::{MultipartForm, Part};
  use axum_test::{TestServer, expect_json};
  use backend::core::error::ApplicationError;
  use backend::features::video::inspect::dto::VideoInspectionResponse;
  use backend::router::routes;
  use serde_json::json;
  use sqlx::PgPool;
  use std::fs;
  use std::os::unix::fs::MetadataExt;

  /// Important
  ///
  /// fs::metadata read files relative to current working directory of running process
  ///
  /// include_bytes! reads files relative source file at compile time

  async fn assert_success_response(
    server: TestServer,
    form: MultipartForm,
    token: String,
    file_name: &str,
    size: u64,
  ) -> Result<(), ApplicationError> {
    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .multipart(form)
      .add_header(header::AUTHORIZATION, token)
      .expect_success()
      .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let video_inspection_response =
      serde_json::from_str::<VideoInspectionResponse>(&response.text())
        .map_err(|err| ApplicationError::Internal(err.to_string()))?;
    assert_eq!(video_inspection_response.original_file_name, file_name);
    assert_eq!(video_inspection_response.file_size_bytes as u64, size);

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_video_process_dual_audio_only(
    pool: PgPool,
  ) -> Result<(), ApplicationError> {
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
      .post(&with_base_route(routes::VIDEO_INSPECT))
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
  async fn test_video_process_dual_broken_truncated(
    pool: PgPool,
  ) -> Result<(), ApplicationError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "broken_truncated.mp4";
    let bearer_token = get_authorization_token(&server).await;
    let file_bytes: &[u8] =
      include_bytes!("./fixtures/media/broken_truncated.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("audio/x-m4a");
    let form = MultipartForm::new()
      .add_part("audio", part_bytes)
      .add_text("operation", "compress");

    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
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
  async fn test_video_process_dual_audio_tracks(
    pool: PgPool,
  ) -> Result<(), ApplicationError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "dual_audio_tracks.mp4";
    let bearer_token = get_authorization_token(&server).await;
    let video_file_meta =
      fs::metadata("./tests/fixtures/media/dual_audio_tracks.mp4")?;
    let file_bytes: &[u8] =
      include_bytes!("./fixtures/media/dual_audio_tracks.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new()
      .add_part("video", part_bytes)
      .add_text("operation", "compress");

    assert_success_response(
      server,
      form,
      bearer_token,
      file_name,
      video_file_meta.size(),
    )
    .await?;

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_video_process_dual_sample_av(
    pool: PgPool,
  ) -> Result<(), ApplicationError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "sample_av.mp4";
    let bearer_token = get_authorization_token(&server).await;
    let video_file_meta = fs::metadata("./tests/fixtures/media/sample_av.mp4")?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/sample_av.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new()
      .add_part("video", part_bytes)
      .add_text("operation", "compress");

    assert_success_response(
      server,
      form,
      bearer_token,
      file_name,
      video_file_meta.size(),
    )
    .await?;

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_video_process_dual_vertical_no_audio(
    pool: PgPool,
  ) -> Result<(), ApplicationError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "vertical_no_audio.mp4";
    let bearer_token = get_authorization_token(&server).await;
    let video_file_meta =
      fs::metadata("./tests/fixtures/media/vertical_no_audio.mp4")?;
    let file_bytes: &[u8] =
      include_bytes!("./fixtures/media/vertical_no_audio.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new()
      .add_part("video", part_bytes)
      .add_text("operation", "compress");

    assert_success_response(
      server,
      form,
      bearer_token,
      file_name,
      video_file_meta.size(),
    )
    .await?;

    Ok(())
  }
}
