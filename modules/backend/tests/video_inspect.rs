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

  async fn assert_error_response(
    server: TestServer,
    form: MultipartForm,
    status: StatusCode,
    message: &str,
  ) -> Result<(), ServerError> {
    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .multipart(form)
      .add_header(header::AUTHORIZATION, "temporary-disabled")
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), status);
    assert_eq!(
      serde_json::from_str::<ErrorBody>(&response.text())?,
      ErrorBody {
        message: message.to_string()
      }
    );

    Ok(())
  }

  fn video_form(file_bytes: &[u8], file_name: &str) -> MultipartForm {
    let part_bytes = Part::bytes(file_bytes.to_vec())
      .file_name(file_name.to_string())
      .mime_type("video/mp4");
    MultipartForm::new().add_part("video", part_bytes)
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

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_correct_video_file_4(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_name: &str = "3_4_mb.mp4";
    let bearer_token = "temporary-disabled".to_string();
    let video_file_meta = fs::metadata("./tests/fixtures/media/3_4_mb.mp4")?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/3_4_mb.mp4");
    let form = video_form(file_bytes, file_name);

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
  async fn test_fail_wrong_format_user_id_header(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/sample_av.mp4");
    let form = video_form(file_bytes, "sample_av.mp4");

    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .multipart(form)
      .add_header(header::AUTHORIZATION, "temporary-disabled")
      .add_header(X_USER_ID_HEADER, "123")
      .expect_failure()
      .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
      serde_json::from_str::<ErrorBody>(&response.text())?,
      ErrorBody {
        message: "Data error: `X-USER-ID` header has wrong value".to_string()
      }
    );

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_extension_m4a(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/audio_only.m4a");
    let form = video_form(file_bytes, "audio_only.m4a");

    assert_error_response(
      server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: File extension is not supported: m4a",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_extension_webm(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    // Extension is checked before the content is read, so any bytes work here
    let form = video_form(b"webm content", "sample.webm");

    assert_error_response(
      server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: File extension is not supported: webm",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_extension_mov(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let form = video_form(b"mov content", "sample.mov");

    assert_error_response(
      server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: File extension is not supported: mov",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_extension_txt(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let form = video_form(b"plain text notes", "notes.txt");

    assert_error_response(
      server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: File extension is not supported: txt",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_empty_file(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let form = video_form(&[], "empty.mp4");

    assert_error_response(
      server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Form data is empty",
    )
    .await
  }

  /// Pins current behavior: invalid media is reported as 500 (test report issue #1)
  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_random_bytes_mp4(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_bytes: Vec<u8> = (0..200_000u32).map(|i| (i * 31 % 251) as u8).collect();
    let form = video_form(&file_bytes, "fake.mp4");

    assert_error_response(
      server,
      form,
      StatusCode::INTERNAL_SERVER_ERROR,
      "Internal server error",
    )
    .await
  }

  /// Pins current behavior: invalid media is reported as 500 (test report issue #1)
  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_text_content_as_mp4(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let form = video_form(b"plain text notes", "notes.mp4");

    assert_error_response(
      server,
      form,
      StatusCode::INTERNAL_SERVER_ERROR,
      "Internal server error",
    )
    .await
  }

  /// Pins current behavior: files without a video stream are accepted (test report issue #2)
  #[sqlx::test(fixtures("create_user"))]
  async fn test_audio_only_content_as_mp4(pool: PgPool) -> Result<(), ServerError> {
    let router = setup_router(pool)?;
    let server = TestServer::new(router);
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/audio_only.m4a");
    let form = video_form(file_bytes, "audio_only.mp4");

    let response = server
      .post(&with_base_route(routes::VIDEO_INSPECT))
      .multipart(form)
      .add_header(header::AUTHORIZATION, "temporary-disabled")
      .add_header(X_USER_ID_HEADER, MOCK_USER_ID)
      .expect_success()
      .await;

    let video_inspection_response =
      serde_json::from_str::<VideoInspectionResponse>(&response.text())?;
    assert!(video_inspection_response.video_streams.is_empty());
    assert!(!video_inspection_response.audio_streams.is_empty());

    Ok(())
  }
}
