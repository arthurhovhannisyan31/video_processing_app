use crate::core::app_state::AppState;
use crate::core::error::ApplicationError;
use crate::features::auth::middleware::auth;
use crate::features::video::configs::DEFAULT_VIDEO_BODY_LIMIT_BYTES;
use crate::features::video::read_video::read_video;
use crate::router::routes;

use crate::features::video::ffprobe_mapper::ffprobe_mapper;
use crate::features::video::ffprobe_runner::ffprobe_runner;
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router, middleware};
use serde_json::json;
use tempfile::TempDir;

pub fn get_video_router(app_state: AppState) -> Router<AppState> {
  Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .layer(DefaultBodyLimit::max(DEFAULT_VIDEO_BODY_LIMIT_BYTES))
    .layer(middleware::from_fn_with_state(app_state, auth))
}

pub async fn inspect_video(
  media_data: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  let temp_dir = TempDir::new().map_err(|err| {
    ApplicationError::Internal(format!(
      "Failed to create temp directory: {err}"
    ))
  })?;

  let file_path = read_video(media_data, temp_dir.path()).await?;
  let inspection_data = ffprobe_runner(file_path).await?;
  println!("inspection_data: {inspection_data:#?}");
  let mapped_data = ffprobe_mapper(inspection_data)?;

  Ok(Json(json!(mapped_data)))
}
