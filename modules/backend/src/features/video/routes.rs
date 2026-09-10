use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use axum::extract::ws::WebSocket;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{any, post};
use axum::{Json, Router};
use serde_json::json;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::core::app_config::AppConfig;
use crate::core::app_state::AppState;
use crate::core::error::{ApplicationError, ServerError};
use crate::core::extractors::XUserIdExtractor;
use crate::features::video::inspect::dto::VideoInspectionResponse;
use crate::features::video::state::VideoState;
use crate::router::routes;

pub fn get_video_router(app_state: AppState) -> Result<Router<AppState>, ServerError> {
  let mut router = Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .route(routes::VIDEO_JOBS, post(process_video))
    .route(routes::VIDEO_WEB_SOCKET_BY_ID, any(websocket_handler))
    .layer(DefaultBodyLimit::max(
      app_state.app_config.video_max_body_size,
    ));

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
pub async fn inspect_video(
  State(app_config): State<Arc<AppConfig>>,
  State(video_state): State<Arc<VideoState>>,
  media_data: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  let media_meta_data = video_state
    .video_service
    .inspect(media_data, app_config.video_inspect_timeout)
    .await?;

  Ok(Json(json!(VideoInspectionResponse::from(media_meta_data))))
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
pub async fn process_video(
  State(app_config): State<Arc<AppConfig>>,
  State(video_state): State<Arc<VideoState>>,
  XUserIdExtractor(user_id): XUserIdExtractor,
  media_data: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  Ok(
    video_state
      .video_service
      .process(
        media_data,
        user_id,
        app_config.video_inspect_timeout,
        app_config.video_process_timeout,
      )
      .await?,
  )
}

#[utoipa::path(
  get,
  path = "/video/ws/{user_id}",
  responses(
    (
      status = 101,
      description = "Switching Protocols to WebSocket.",
      headers(
          ("Upgrade" = String, description = "websocket"),
          ("Connection" = String, description = "Upgrade")
      )
    )
  )
)]
async fn websocket_handler(
  State(video_state): State<Arc<VideoState>>,
  Path(user_id): Path<Uuid>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApplicationError> {
  Ok(
    ws.on_failed_upgrade(move |err| {
      error!(user_id = %user_id, error = %err, "Error upgrading websocket for user");
    })
    .on_upgrade(move |socket| handle_socket(socket, user_id, video_state)),
  )
}

async fn handle_socket(socket: WebSocket, user_id: Uuid, video_state: Arc<VideoState>) {
  let _ = video_state
    .video_service
    .handle_socket(socket, user_id)
    .await;
}
