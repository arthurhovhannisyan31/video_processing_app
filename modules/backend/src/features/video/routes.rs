use axum::extract::{DefaultBodyLimit, Multipart};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router, middleware};
use serde_json::json;
use tempfile::TempDir;
use utoipa::ToSchema;

use crate::core::app_state::AppState;
use crate::core::error::{ApplicationError, ServerError};
use crate::features::auth::middleware::auth;
use crate::features::video::constants::DEFAULT_VIDEO_BODY_LIMIT_BYTES;
use crate::features::video::helpers::append_path_suffix;
use crate::features::video::inspect::dto::VideoInspectionResponse;
use crate::features::video::inspect::ffprobe_mapper::ffprobe_mapper;
use crate::features::video::inspect::ffprobe_runner::ffprobe_runner;
use crate::features::video::process::configs::{OUTPUT_PATH_SUFFIX, get_preset_by_name};
use crate::features::video::process::ffmpeg_runner::ffmpeg_runner;
use crate::features::video::process::types::ProcessVideoMeta;
use crate::features::video::{inspect, process};
use crate::router::routes;

pub fn get_video_router(app_state: AppState) -> Router<AppState> {
  Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .route(routes::VIDEO_JOBS, post(process_video))
    .layer(DefaultBodyLimit::max(DEFAULT_VIDEO_BODY_LIMIT_BYTES))
    .layer(middleware::from_fn_with_state(app_state, auth))
}

// Struct used for openapi schema typings
#[allow(unused)]
#[derive(ToSchema)]
pub struct InspectVideoPayload {
  #[schema(value_type = String, format = Binary)]
  pub file: Vec<u8>,
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
  pub file: Vec<u8>,
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

  Ok("Success")
}
