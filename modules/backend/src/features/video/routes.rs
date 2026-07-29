use crate::core::app_state::AppState;
use crate::core::error::ApplicationError;
use crate::features::auth::middleware::auth;
use crate::router::routes;

use axum::extract::Multipart;
use axum::routing::post;
use axum::{Json, Router, middleware};
use serde_json::{Value, json};

pub fn get_video_router(app_state: AppState) -> Router<AppState> {
  Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .layer(middleware::from_fn_with_state(app_state, auth))
}

pub async fn inspect_video(
  mut multipart: Multipart,
) -> Result<Json<Value>, ApplicationError> {
  println!("Hello inspect video");

  while let Some(field) = multipart.next_field().await? {
    println!("field {:#?}", field);
  }

  // add input dto
  // validate input
  // read input data with ffprobe

  Ok(Json(json!({
    "status": "ok",
  })))
}
