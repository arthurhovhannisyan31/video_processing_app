use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{DefaultBodyLimit, Multipart, Path, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{any, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tempfile::TempDir;
use tokio::sync::mpsc;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tracing::{error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::core::app_state::AppState;
use crate::core::error::{ApplicationError, ServerError};
use crate::core::extractors::XUserIdExtractor;
use crate::features::video::helpers::append_path_suffix;
use crate::features::video::inspect::dto::VideoInspectionResponse;
use crate::features::video::process::configs::{OUTPUT_PATH_SUFFIX, get_preset_by_name};
use crate::features::video::process::types::ProcessVideoMeta;
use crate::features::video::state::{VideoState, VideoStateMessage};
use crate::features::video::{inspect, process};
use crate::router::routes;

pub fn get_video_router(app_state: AppState) -> Result<Router<AppState>, ServerError> {
  let mut router = Router::new()
    .route(routes::VIDEO_INSPECT, post(inspect_video))
    .route(routes::VIDEO_JOBS, post(process_video))
    .route(routes::VIDEO_WEB_SOCKET_BY_ID, any(video_ws))
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
  let inspect_meta = inspect::form_data_reader::read(media_data, temp_dir.path()).await?;
  let inspection_data = inspect::ffprobe_runner::inspect_file(&inspect_meta.local_path).await?;
  let mapped_data = inspect::ffprobe_mapper::map_media_meta(inspection_data)?;

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
pub async fn process_video(
  State(video_state): State<Arc<VideoState>>,
  XUserIdExtractor(user_id): XUserIdExtractor,
  media_data: Multipart,
) -> Result<impl IntoResponse, ApplicationError> {
  let temp_dir = TempDir::new().map_err(ServerError::IO)?;
  let ProcessVideoMeta {
    operation,
    local_path,
    file_name,
  } = process::form_data_reader::read(media_data, temp_dir.path()).await?;
  let output_path = append_path_suffix(&local_path, OUTPUT_PATH_SUFFIX)?;
  let preset = get_preset_by_name(&operation)?;
  let duration = process::ffprobe_runner::inspect_file_duration(&local_path).await?;
  process::ffmpeg_runner::process_file(
    &local_path,
    &output_path,
    &file_name,
    preset,
    video_state,
    user_id,
    duration,
  )
  .await?;

  Ok(process::build_response::build_response(&local_path, &output_path).await?)
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
async fn video_ws(
  State(video_state): State<Arc<VideoState>>,
  Path(user_id): Path<Uuid>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApplicationError> {
  // TODO on_failed_upgrade

  Ok(ws.on_upgrade(move |socket| handle_socket(socket, user_id, video_state)))
}

async fn handle_socket(socket: WebSocket, user_id: Uuid, video_state: Arc<VideoState>) {
  let (mut sink, mut stream) = socket.split();
  let (tx, mut rx) = mpsc::channel::<VideoStateMessage>(10);

  {
    let mut connections_map = video_state.connections_map.write();
    if connections_map.contains_key(&user_id) {
      warn!("Conflicting key for video WS connections map: {user_id}");
      return;
    }
    connections_map.insert(user_id, tx);
  }

  loop {
    tokio::select! {
      progress_msg = rx.recv() => {
        match progress_msg{
          Some(msg) => {
            let message = Message::from(json!(msg.message).to_string());

            if let Err(err) = sink.send(message).await {
              error!("Failed to send message to: {user_id}. Err: {err}");
              break; // Disconnect if the socket is broken
            }
          }
          None => {
            let mut connections_map = video_state.connections_map.write();
            connections_map.remove(&user_id);
            break;
          }
        }
      }
      client_msg = stream.next() => {
        match client_msg{
          // Ignore regular incoming messages: Ping, Pong, Close
          Some(Ok(msg)) => {
            info!("Regular message:  {user_id} {msg:?}");
          }
          Some(Err(err)) => {
            error!("WebSocket error for user {user_id}: {err}");
            break;
          }
          None => {
            info!("WebSocket connection has been closed by user {user_id}");
            let mut connections_map = video_state.connections_map.write();
            connections_map.remove(&user_id);

            break;
          }
        }
      }
    }
  }
}
