use crate::core::app_state::AppState;
use crate::core::error::ApplicationError;
use crate::features::auth::middleware::auth;
use crate::features::video::constants::DEFAULT_VIDEO_BODY_LIMIT_BYTES;
use crate::features::video::read_to_file::read_to_file;
use crate::router::routes;

use crate::features::video::ffprobe_runner::ffprobe_runner;
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router, middleware};

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
  let file_path = read_to_file(multipart).await?;
  let json_data = ffprobe_runner(file_path).await?;

  Ok(Json(json_data))
}
