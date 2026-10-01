use std::collections::HashMap;
use std::collections::hash_map::Entry;
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
use tracing::{error, warn};
use uuid::Uuid;

use crate::core::error::ServerError;
use crate::core::hash::calculate_hash;
use crate::features::video::cache::{MediaDataCache, build_media_data_cache};
use crate::features::video::helpers::{append_path_suffix, get_file_duration};
use crate::features::video::model::{MediaMetadata, VideoStateProgress};
use crate::features::video::process::configs::OUTPUT_PATH_SUFFIX;
use crate::features::video::process::helpers::get_args;
use crate::features::video::process::types::ProcessVideoMeta;
use crate::features::video::{inspect, process};

pub type VideoWsConnectionsMap = RwLock<HashMap<Uuid, mpsc::Sender<VideoStateProgress>>>;

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
    user_id: Uuid,
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
    let file_hash = calculate_hash(&format!("{}{}", inspect_meta.file_name, user_id));
    self
      .media_data_cache
      .insert(file_hash, media_meta_data.clone());

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
      user_id,
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

    let mut has_conflict = false;
    {
      let mut connections_map = self.connections_map.write();
      match connections_map.entry(user_id) {
        Entry::Vacant(v) => {
          v.insert(tx);
        }
        Entry::Occupied(_o) => {
          has_conflict = true;
          warn!(user_id = %user_id, "Conflicting key for video WS connections map");
        }
      }
    }

    if has_conflict {
      if let Err(err) = send_close_message(
        &mut sink,
        close_code::NORMAL,
        "Duplicated connections are not allowed: Closing channel",
      )
      .await
      {
        error!(error = %err, "Failed sending ws close message");
      }

      return;
    }

    loop {
      tokio::select! {
        progress_msg = rx.recv() => {
          if let Some(msg) = progress_msg {
            let message = Message::from(json!(msg).to_string());

            if let Err(err) = sink.send(message).await {
              error!(user_id = %user_id, error = %err, "Failed to send message to user");

              break; // Disconnect if the socket is broken
            }
          }
          // Err case is not needed. Sender is not dropped by ffmpeg runner so channel only closed
          // when removed from connections_map.
        }
        client_msg = stream.next() => {
          match client_msg{
            // Ping, Pong, Close are handled
            Some(Ok(_msg)) => {}
            // Connection reset without closing handshake
            Some(Err(err)) => {
              warn!(user_id = %user_id, err = %err, "Error receiving message from user");

              if let Err(err) = send_close_message(&mut sink, close_code::ERROR, "Error occurred: Closing channel").await {
                error!(error = %err, "Failed sending ws close message");
              }

              break;
            }
            // Connection closed
            None => {
              warn!(user_id = %user_id, "WebSocket connection has been closed by user");
              break;
            }
          }
        }
      }
    }

    let mut connections_map = self.connections_map.write();
    connections_map.remove(&user_id);
  }
}
async fn send_close_message(
  socket_sink: &mut SplitSink<WebSocket, Message>,
  code: u16,
  reason: &str,
) -> Result<(), ServerError> {
  socket_sink
    .send(Message::Close(Some(CloseFrame {
      code,
      reason: reason.into(),
    })))
    .await?;

  Ok(())
}
