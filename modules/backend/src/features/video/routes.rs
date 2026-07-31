use crate::core::app_state::AppState;
use crate::core::error::ApplicationError;
use crate::features::auth::middleware::auth;
use crate::features::video::configs::DEFAULT_VIDEO_BODY_LIMIT_BYTES;
use crate::features::video::ffprobe_runner::ffprobe_runner;
use crate::features::video::read_video::read_video;
use crate::router::routes;

use axum::extract::{DefaultBodyLimit, Multipart};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router, middleware};
use tempfile::TempDir;

pub fn get_video_router(app_state: AppState) -> Router<AppState> {
  Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .layer(middleware::from_fn_with_state(app_state, auth))
    .layer(DefaultBodyLimit::max(DEFAULT_VIDEO_BODY_LIMIT_BYTES))
  // TODO Add upload timeout middleware (long upload)
  // TODO Add upload idle middleware (interrupted upload)
}

pub async fn inspect_video(
  multipart: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  // TODO add input dto
  // TODO validate input

  // use temp_dir for file cleanup after processing
  let temp_dir = TempDir::new().map_err(|err| {
    ApplicationError::Internal(format!(
      "Failed to create temp directory: {err}"
    ))
  })?;

  println!("temp_dir: {temp_dir:?}");

  let file_path = read_video(multipart, temp_dir.path()).await?;
  let json_data = ffprobe_runner(file_path).await?;

  Ok(Json(json_data))
}
