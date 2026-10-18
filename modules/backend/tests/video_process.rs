mod common;
use common::utils;

#[cfg(test)]
mod test_video_process_api {
  use axum::http::StatusCode;
  use axum_test::TestServer;
  use axum_test::multipart::{MultipartForm, Part};
  use sqlx::PgPool;
  use video_processing_server::core::error::{ErrorBody, ServerError};
  use video_processing_server::router::routes;

  use crate::common::constants::DEFAULT_FORM_DATA_ARGS;
  use crate::common::utils::setup_server;
  use crate::utils::{setup_router, with_base_route};

  /// Important
  ///
  /// fs::metadata read files relative to current working directory of running process
  ///
  /// include_bytes! reads files relative source file at compile time
  async fn assert_success_response(
    server: &TestServer,
    form: MultipartForm,
  ) -> Result<(), ServerError> {
    let response = server
      .post(&with_base_route(routes::VIDEO_JOBS))
      .multipart(form)
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
    assert!(!response.as_bytes().is_empty());

    Ok(())
  }

  async fn assert_error_response(
    server: &TestServer,
    form: MultipartForm,
    status: StatusCode,
    message: &str,
  ) -> Result<(), ServerError> {
    let response = server
      .post(&with_base_route(routes::VIDEO_JOBS))
      .multipart(form)
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

  fn with_args(form: MultipartForm, args: &[(&str, &str)]) -> MultipartForm {
    args
      .iter()
      .fold(form, |form, (name, value)| form.add_text(name, value))
  }

  /// Default arguments without the given fields
  fn args_without(names: &[&str]) -> Vec<(&'static str, &'static str)> {
    DEFAULT_FORM_DATA_ARGS
      .into_iter()
      .filter(|(name, _)| !names.contains(name))
      .collect()
  }

  /// Default arguments with one field value replaced
  fn args_with<'a>(field: &str, value: &'a str) -> Vec<(&'a str, &'a str)> {
    DEFAULT_FORM_DATA_ARGS
      .into_iter()
      .map(|(name, default)| (name, if name == field { value } else { default }))
      .collect()
  }

  /// Video form with all default processing arguments
  fn process_form(file_bytes: &[u8], file_name: &str) -> MultipartForm {
    with_args(video_form(file_bytes, file_name), &DEFAULT_FORM_DATA_ARGS)
  }

  fn sample_form(args: &[(&str, &str)]) -> MultipartForm {
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/sample_av.mp4");
    with_args(video_form(file_bytes, "sample_av.mp4"), args)
  }

  // Form structure

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_wrong_file_format(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/audio_only.m4a");
    let part_bytes = Part::bytes(file_bytes)
      .file_name("audio_only.m4a")
      .mime_type("audio/x-m4a");
    let form = with_args(
      MultipartForm::new().add_part("audio", part_bytes),
      &DEFAULT_FORM_DATA_ARGS,
    );

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Field name is not supported: audio",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_text_field(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = sample_form(&DEFAULT_FORM_DATA_ARGS).add_text("bitrate", "128k");

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Field name is not supported: bitrate",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_missing_video_field(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = with_args(MultipartForm::new(), &DEFAULT_FORM_DATA_ARGS);

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Missing 'video' field",
    )
    .await
  }

  // Processing arguments: missing fields

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_missing_operation(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = sample_form(&args_without(&["operation"]));

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Missing 'operation' field",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_missing_codec(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    // Preset is validated against codec, so both are left out to reach the missing codec check
    let form = sample_form(&args_without(&["codec", "preset"]));

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Missing 'codec' field",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_missing_preset(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = sample_form(&args_without(&["preset"]));

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Missing 'preset' field",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_missing_quality(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = sample_form(&args_without(&["quality"]));

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Missing 'quality' field",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_preset_before_codec(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = sample_form(&[
      ("operation", "compress"),
      ("preset", "medium"),
      ("codec", "libx264"),
      ("quality", "23"),
    ]);

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Missing 'codec' field, ensure it is listed before 'preset' field",
    )
    .await
  }

  // Processing arguments: invalid values

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_operation(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = sample_form(&args_with("operation", "explode"));

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Operation is not supported: explode",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_codec(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = sample_form(&args_with("codec", "libx265"));

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Codec is not supported: libx265",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_preset(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    // Valid x264 preset in ffmpeg, but not in the allowed list
    let form = sample_form(&args_with("preset", "ultrafast"));

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Preset is not supported: ultrafast",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_quality(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = sample_form(&args_with("quality", "22"));

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Quality is not supported: 22",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_empty_argument_values(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let cases = [
      ("operation", "Data error: Operation is missing"),
      ("codec", "Data error: Codec is missing"),
      ("preset", "Data error: Preset is missing"),
      ("quality", "Data error: Quality is missing"),
    ];

    for (field, message) in cases {
      let form = sample_form(&args_with(field, ""));
      assert_error_response(&server, form, StatusCode::BAD_REQUEST, message).await?;
    }

    Ok(())
  }

  // File validation

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_extension_m4a(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/audio_only.m4a");
    let form = process_form(file_bytes, "audio_only.m4a");

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: File extension is not supported: m4a",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_extension_webm(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    // Extension is checked before the content is read, so any bytes work here
    let form = process_form(b"webm content", "sample.webm");

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: File extension is not supported: webm",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_extension_mov(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = process_form(b"mov content", "sample.mov");

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: File extension is not supported: mov",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_unsupported_extension_txt(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = process_form(b"plain text notes", "notes.txt");

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: File extension is not supported: txt",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_empty_file(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = process_form(&[], "empty.mp4");

    assert_error_response(
      &server,
      form,
      StatusCode::BAD_REQUEST,
      "Data error: Form data is empty",
    )
    .await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_broken_video_file(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/broken_truncated.mp4");
    let form = process_form(file_bytes, "broken_truncated.mp4");

    assert_error_response(
      &server,
      form,
      StatusCode::INTERNAL_SERVER_ERROR,
      "Internal server error",
    )
    .await
  }

  /// Pins current behavior: invalid media is reported as 500 (test report issue #1)
  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_random_bytes_mp4(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: Vec<u8> = (0..200_000u32).map(|i| (i * 31 % 251) as u8).collect();
    let form = process_form(&file_bytes, "fake.mp4");

    assert_error_response(
      &server,
      form,
      StatusCode::INTERNAL_SERVER_ERROR,
      "Internal server error",
    )
    .await
  }

  /// Pins current behavior: invalid media is reported as 500 (test report issue #1)
  #[sqlx::test(fixtures("create_user"))]
  async fn test_fail_text_content_as_mp4(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let form = process_form(b"plain text notes", "notes.mp4");

    assert_error_response(
      &server,
      form,
      StatusCode::INTERNAL_SERVER_ERROR,
      "Internal server error",
    )
    .await
  }

  // Success

  /// Pins current behavior: audio-only input is compressed and labelled as video/mp4
  /// (test report issue #2)
  #[sqlx::test(fixtures("create_user"))]
  async fn test_audio_only_content_as_mp4(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/audio_only.m4a");
    let form = process_form(file_bytes, "audio_only.mp4");

    assert_success_response(&server, form).await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_correct_video_file_1(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/dual_audio_tracks.mp4");
    let form = process_form(file_bytes, "dual_audio_tracks.mp4");

    assert_success_response(&server, form).await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_correct_video_file_2(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/sample_av.mp4");
    let form = process_form(file_bytes, "sample_av.mp4");

    assert_success_response(&server, form).await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_correct_video_file_3(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/vertical_no_audio.mp4");
    let form = process_form(file_bytes, "vertical_no_audio.mp4");

    assert_success_response(&server, form).await
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_all_x264_presets(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;

    for preset in ["superfast", "medium", "slow"] {
      assert_success_response(&server, sample_form(&args_with("preset", preset))).await?;
    }

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_all_qualities(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;

    for quality in ["18", "23", "28"] {
      assert_success_response(&server, sample_form(&args_with("quality", quality))).await?;
    }

    Ok(())
  }

  #[sqlx::test(fixtures("create_user"))]
  async fn test_success_field_order(pool: PgPool) -> Result<(), ServerError> {
    let server = setup_server(pool).await?;
    let file_bytes: &[u8] = include_bytes!("./fixtures/media/sample_av.mp4");
    // Arguments before the file; only 'codec' before 'preset' is required
    let form = with_args(
      MultipartForm::new(),
      &[
        ("quality", "23"),
        ("codec", "libx264"),
        ("operation", "compress"),
        ("preset", "medium"),
      ],
    )
    .add_part(
      "video",
      Part::bytes(file_bytes)
        .file_name("sample_av.mp4")
        .mime_type("video/mp4"),
    );

    assert_success_response(&server, form).await
  }
}
