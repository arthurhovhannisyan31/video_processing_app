mod constants;
mod utils;

#[cfg(test)]
mod test_video_inspect_api {
  use std::fs;
  use std::os::unix::fs::MetadataExt;

  use axum::http::{StatusCode, header};
  use axum_test::TestServer;
  use axum_test::multipart::{MultipartForm, Part};
  use sqlx::PgPool;
  use video_processing_server::core::error::{ErrorBody, ServerError};
  use video_processing_server::core::extractors::X_USER_ID_HEADER;
  use video_processing_server::features::video::inspect::dto::VideoInspectionResponse;
  use video_processing_server::router::routes;

  use crate::constants::MOCK_USER_ID;
  use crate::utils::{setup_router, with_base_route};

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
  ) -> Result<(), ServerError> {
    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .multipart(form)
      .add_header(header::AUTHORIZATION, token)
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .expect_success()
      .await;

    response.assert_status_ok();

    let video_inspection_response =
      serde_json::from_str::<VideoInspectionResponse>(&response.text())?;
    assert_eq!(video_inspection_response.original_file_name, file_name);
    assert_eq!(video_inspection_response.file_size_bytes as u64, size);

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_missing_user_id_header(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name: &str = "dual_audio_tracks.mp4";
    let bearer_token = "temporary-disabled".to_string();
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/dual_audio_tracks.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new().add_part("video", part_bytes);

    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .add_header("X-Forwarded-For", "127.0.0.1")
      .multipart(form)
      .add_header(header::AUTHORIZATION, bearer_token)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
      serde_json::from_str::<ErrorBody>(&response.text())?,
      ErrorBody {
        message: "Data error: `X-USER-ID` header is missing".to_string()
      }
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_field(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "audio_only.m4a";
    let bearer_token = "temporary-disabled";
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/audio_only.m4a");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("audio/x-m4a");
    let form = MultipartForm::new().add_part("audio", part_bytes);
    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .add_header("X-Forwarded-For", "127.0.0.1")
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .multipart(form)
      .add_header(header::AUTHORIZATION, bearer_token)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
      serde_json::from_str::<ErrorBody>(&response.text())?,
      ErrorBody {
        message: "Data error: Field name is not supported: audio".to_string()
      }
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_broken_video_file(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name = "broken_truncated.mp4";
    let bearer_token = "temporary-disabled";
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/broken_truncated.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new().add_part("video", part_bytes);
    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .multipart(form)
      .add_header(header::AUTHORIZATION, bearer_token)
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
      serde_json::from_str::<ErrorBody>(&response.text())?,
      ErrorBody {
        message: "Internal server error".to_string()
      }
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_missing_video_field(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let bearer_token = "temporary-disabled";
    let form = MultipartForm::new();

    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .multipart(form)
      .add_header(header::AUTHORIZATION, bearer_token)
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
      serde_json::from_str::<ErrorBody>(&response.text())?,
      ErrorBody {
        message: "Multipart form error: Error parsing `multipart/form-data` request".to_string()
      }
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_correct_video_file_1(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name: &str = "dual_audio_tracks.mp4";
    let bearer_token = "temporary-disabled".to_string();
    let video_file_meta = fs::metadata("./tests/fixtures/media/dual_audio_tracks.mp4")?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/dual_audio_tracks.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new().add_part("video", part_bytes);

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
  async fn test_success_correct_video_file_2(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name: &str = "sample_av.mp4";
    let bearer_token = "temporary-disabled".to_string();
    let video_file_meta = fs::metadata("./tests/fixtures/media/sample_av.mp4")?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/sample_av.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new().add_part("video", part_bytes);

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
  async fn test_success_correct_video_file_3(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name: &str = "vertical_no_audio.mp4";
    let bearer_token = "temporary-disabled".to_string();
    let video_file_meta = fs::metadata("./tests/fixtures/media/vertical_no_audio.mp4")?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/vertical_no_audio.mp4");
    let part_bytes = Part::bytes(file_bytes)
      .file_name(file_name)
      .mime_type("video/mp4");
    let form = MultipartForm::new().add_part("video", part_bytes);

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
