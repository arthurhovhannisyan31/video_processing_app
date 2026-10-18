use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::WebSocket;
use axum::extract::{DefaultBodyLimit, Multipart, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{any, post};
use axum::{Json, Router};
use axum_session::Session;
use axum_session_sqlx::SessionPgPool;
use serde_json::json;
use tower_governor::GovernorLayer;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tracing::error;
use utoipa::ToSchema;

use crate::core::app_config::AppConfig;
use crate::core::app_state::AppState;
use crate::core::error::{ApplicationError, ServerError};
use crate::core::governor::build_rate_limiter_config;
use crate::features::video::inspect::dto::VideoInspectionResponse;
use crate::features::video::state::VideoState;
use crate::router::routes;

pub fn get_video_router(app_state: AppState) -> Result<Router<AppState>, ServerError> {
  let mut router = Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .route(routes::VIDEO_JOBS, post(process_video))
    .route(routes::VIDEO_WEB_SOCKET, any(websocket_handler));

  if app_state.app_config.is_production {
    let governor_conf = build_rate_limiter_config(
      SmartIpKeyExtractor,
      Some(Duration::from_secs(
        app_state.app_config.video.rate_limit_period,
      )),
      Some(app_state.app_config.video.rate_limit_size),
    )?;

    {
      let mut rate_limiters = app_state.system_state.rate_limiters.lock();
      rate_limiters.push(governor_conf.limiter().clone())
    }

    router = router.layer(GovernorLayer::new(governor_conf));
  }

  router = router.layer(DefaultBodyLimit::max(
    app_state.app_config.video.max_body_size,
  ));

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
  session: Session<SessionPgPool>,
  media_data: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  let session_id = session.get_session_id();

  let media_meta_data = video_state
    .video_service
    .inspect(media_data, session_id, app_config.video.inspect_timeout)
    .await?;

  Ok(Json(json!(VideoInspectionResponse::from(media_meta_data))))
}

// Struct used for openapi schema typings
// TODO Replace strings with enums and check spec result
#[allow(unused)]
#[derive(ToSchema)]
pub struct ProcessVideoPayload {
  pub operation: String,
  pub codec: String,
  pub preset: String,
  pub quality: String,
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
  session: Session<SessionPgPool>,
  media_data: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  let session_id = session.get_session_id();

  Ok(
    video_state
      .video_service
      .process(
        media_data,
        session_id,
        app_config.video.inspect_timeout,
        app_config.video.process_timeout,
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
pub async fn websocket_handler(
  State(video_state): State<Arc<VideoState>>,
  session: Session<SessionPgPool>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  let session_id = session.get_session_id();
  let session_id_clone = session_id.clone();

  ws.on_failed_upgrade(move |err| {
    error!(session_id = %session_id_clone, error = %err, "Error upgrading websocket for user");
  })
  .on_upgrade(move |socket| handle_socket(socket, session_id, video_state))
}

async fn handle_socket(socket: WebSocket, session_id: String, video_state: Arc<VideoState>) {
  let _ = video_state
    .video_service
    .handle_socket(socket, session_id)
    .await;
}
