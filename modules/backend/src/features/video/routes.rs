use crate::core::app_state::AppState;
use crate::core::error::ApplicationError;
use crate::features::auth::middleware::auth;
use crate::features::video::configs::DEFAULT_VIDEO_BODY_LIMIT_BYTES;
use crate::features::video::read_video::read_video;
use crate::router::routes;

use axum::extract::{DefaultBodyLimit, Multipart};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router, middleware};
use serde_json::json;

pub fn get_video_router(app_state: AppState) -> Router<AppState> {
  Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .layer(DefaultBodyLimit::max(DEFAULT_VIDEO_BODY_LIMIT_BYTES))
    .layer(middleware::from_fn_with_state(app_state, auth))
}

pub async fn inspect_video(
  media_data: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  let mapped_data = read_video(media_data).await?;

  Ok(Json(json!(mapped_data)))
}
