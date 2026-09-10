use std::collections::HashMap;
use std::io::ErrorKind;
use std::time::Duration;

use axum::body::Body;
use axum::extract::Multipart;
use axum::extract::ws::{CloseFrame, Message, WebSocket, close_code};
use axum::http::Response;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use mini_moka::sync::Cache;
use parking_lot::RwLock;
use serde_json::json;
use tempfile::TempDir;
use tokio::io;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::core::error::ServerError;
use crate::features::video::cache::build_media_data_cache;
use crate::features::video::helpers::{append_path_suffix, get_file_duration};
use crate::features::video::model::{MediaMetadata, VideoStateProgress};
use crate::features::video::process::configs::OUTPUT_PATH_SUFFIX;
use crate::features::video::process::helpers::get_args;
use crate::features::video::process::types::ProcessVideoMeta;
use crate::features::video::{inspect, process};

pub type VideoWsConnectionsMap = RwLock<HashMap<Uuid, mpsc::Sender<VideoStateProgress>>>;
pub type MediaDataCache = Cache<String, MediaMetadata>;

pub struct VideoService {
  pub connections_map: VideoWsConnectionsMap,
  pub media_data_cache: MediaDataCache,
}

impl Default for VideoService {
  fn default() -> Self {
    Self {
      connections_map: RwLock::new(HashMap::new()),
      media_data_cache: build_media_data_cache(),
    }
  }
}

impl VideoService {
  pub async fn inspect(
    &self,
    media_data: Multipart,
    video_inspect_timeout: Duration,
  ) -> Result<MediaMetadata, ServerError> {
    let temp_dir = TempDir::new().map_err(|err| {
      ServerError::IO(io::Error::new(
        ErrorKind::PermissionDenied,
        format!("Failed to create temp directory: {err}"),
      ))
    })?;

    let inspect_meta = inspect::form_data_reader::read(media_data, temp_dir.path()).await?;
    let inspection_raw_data =
      inspect::ffprobe_runner::inspect_file(&inspect_meta.local_path, video_inspect_timeout)
        .await?;
    let media_meta_data = inspect::ffprobe_mapper::map_media_meta(inspection_raw_data)?;

    self
      .media_data_cache
      .insert(inspect_meta.file_name, media_meta_data.clone());

    Ok(media_meta_data)
  }

  pub async fn process(
    &self,
    media_data: Multipart,
    user_id: Uuid,
    video_inspect_timeout: Duration,
    video_process_timeout: Duration,
  ) -> Result<Response<Body>, ServerError> {
    let temp_dir = TempDir::new().map_err(|err| {
      ServerError::IO(io::Error::new(
        ErrorKind::PermissionDenied,
        format!("Failed to create temp directory: {err}"),
      ))
    })?;
    let ProcessVideoMeta {
      operation,
      local_path,
      file_name,
    } = process::form_data_reader::read(media_data, temp_dir.path()).await?;
    let output_path = append_path_suffix(&local_path, OUTPUT_PATH_SUFFIX)?;
    let ffmpeg_args = get_args(&local_path, &output_path, &operation)?;
    let duration = get_file_duration(
      &file_name,
      &local_path,
      &self.media_data_cache,
      video_inspect_timeout,
    )
    .await?;

    process::ffmpeg_runner::process_file(
      ffmpeg_args,
      &file_name,
      &self.connections_map,
      user_id,
      duration,
      video_process_timeout,
    )
    .await?;

    process::build_response::build_response(&local_path, &output_path).await
  }

  pub async fn handle_socket(&self, socket: WebSocket, user_id: Uuid) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::channel::<VideoStateProgress>(10);

    {
      let mut connections_map = self.connections_map.write();
      if connections_map.contains_key(&user_id) {
        warn!(user_id = %user_id, "Conflicting key for video WS connections map");

        return;
      }
      connections_map.insert(user_id, tx);
    }

    loop {
      tokio::select! {
        progress_msg = rx.recv() => {
          match progress_msg{
            Some(msg) => {
              let message = Message::from(json!(msg).to_string());

              if let Err(err) = sink.send(message).await {
                error!(user_id = %user_id, error = %err, "Failed to send message to user");

                break; // Disconnect if the socket is broken
              }
            }
            None => {
              let mut connections_map = self.connections_map.write();
              connections_map.remove(&user_id);

              break;
            }
          }
        }
        client_msg = stream.next() => {
          match client_msg{
            // Ping, Pong, Close are handled
            Some(Ok(msg)) => {
              info!("Regular message:  {user_id} {msg:?}");
            }
            Some(Err(err)) => {
              warn!(user_id = %user_id, err = %err, "Error receiving message from user");
              send_close_message(&mut sink, close_code::ERROR, &format!("Error occured: {}", err)).await;

              break;
            }
            None => {
              info!(user_id = %user_id, "WebSocket connection has been closed by user");
              let mut connections_map = self.connections_map.write();
              connections_map.remove(&user_id);

              break;
            }
          }
        }
      }
    }
  }
}
async fn send_close_message(
  socket_sink: &mut SplitSink<WebSocket, Message>,
  code: u16,
  reason: &str,
) {
  _ = socket_sink
    .send(Message::Close(Some(CloseFrame {
      code,
      reason: reason.into(),
    })))
    .await;
}
