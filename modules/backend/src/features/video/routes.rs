use std::time::Duration;

use anyhow::anyhow;
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::json;
use tempfile::TempDir;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use utoipa::ToSchema;

use crate::core::app_state::AppState;
use crate::core::error::{ApplicationError, ServerError};
use crate::features::video::helpers::append_path_suffix;
use crate::features::video::inspect::dto::VideoInspectionResponse;
use crate::features::video::inspect::ffprobe_mapper::ffprobe_mapper;
use crate::features::video::inspect::ffprobe_runner::ffprobe_runner;
use crate::features::video::process::build_response::build_response;
use crate::features::video::process::configs::{OUTPUT_PATH_SUFFIX, get_preset_by_name};
use crate::features::video::process::ffmpeg_runner::ffmpeg_runner;
use crate::features::video::process::types::ProcessVideoMeta;
use crate::features::video::{inspect, process};
use crate::router::routes;

pub fn get_video_router(app_state: AppState) -> Result<Router<AppState>, ServerError> {
  let mut router = Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .route(routes::VIDEO_JOBS, post(process_video))
    .layer(DefaultBodyLimit::max(
      app_state.app_config.video_max_body_size,
    ));
  // .layer(middleware::from_fn_with_state(app_state, auth));

  if app_state.app_config.is_production {
    let rate_limiter = GovernorConfigBuilder::default()
      .period(Duration::from_secs(
        app_state.app_config.video_rate_limit_period,
      ))
      .burst_size(app_state.app_config.video_rate_limit_size)
      .key_extractor(SmartIpKeyExtractor)
      .finish()
      .ok_or(ServerError::OtherError(anyhow!(
        "Wrong tower_governor configuration"
      )))?;

    router = router.layer(GovernorLayer::new(rate_limiter))
  }

  Ok(router)
}

// Struct used for openapi schema typings
#[allow(unused)]
#[derive(ToSchema)]
pub struct InspectVideoPayload {
  #[schema(value_type = String, format = Binary)]
  pub video: Vec<u8>,
}

#[utoipa::path(
  post,
  path = routes::VIDEO_INSPECT,
  request_body(
      content = InspectVideoPayload,
      content_type = "multipart/form-data"
  ),
  responses(
    (status = OK, description = "Success", body = Object, content_type = "application/json"),
    (status = UNAUTHORIZED, description = "Unauthorized"),
    (status = INTERNAL_SERVER_ERROR, description = "Server internal error", body = Object, content_type = "application/json")
  )
)]
pub async fn inspect_video(media_data: Multipart) -> Result<impl IntoResponse, ApplicationError> {
  let temp_dir = TempDir::new()
    .map_err(|err| ApplicationError::Internal(format!("Failed to create temp directory: {err}")))?;
  let file_path = inspect::form_data_reader::read(media_data, temp_dir.path()).await?;
  let inspection_data = ffprobe_runner(&file_path).await?;
  let mapped_data = ffprobe_mapper(inspection_data)?;

  Ok(Json(json!(VideoInspectionResponse::from(mapped_data))))
}

// Struct used for openapi schema typings
#[allow(unused)]
#[derive(ToSchema)]
pub struct ProcessVideoPayload {
  pub operation: String,
  #[schema(value_type = String, format = Binary)]
  pub video: Vec<u8>,
}

#[utoipa::path(
  post,
  path = routes::VIDEO_JOBS,
  request_body(
      content = ProcessVideoPayload,
      content_type = "multipart/form-data"
  ),
  responses(
    (status = OK, description = "Success", body = Object, content_type = "application/json"),
    (status = UNAUTHORIZED, description = "Unauthorized"),
    (status = INTERNAL_SERVER_ERROR, description = "Server internal error", body = Object, content_type = "application/json")
  )
)]
pub async fn process_video(media_data: Multipart) -> Result<impl IntoResponse, ApplicationError> {
  let temp_dir = TempDir::new().map_err(ServerError::IO)?;
  let ProcessVideoMeta { command, file_path } =
    process::form_data_reader::read(media_data, temp_dir.path()).await?;
  let output_path = append_path_suffix(&file_path, OUTPUT_PATH_SUFFIX)?;
  let preset = get_preset_by_name(&command)?;
  ffmpeg_runner(&file_path, &output_path, preset).await?;

  Ok(build_response(&file_path, &output_path).await?)
}
