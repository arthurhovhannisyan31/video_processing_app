use std::sync::Arc;

use axum::extract::FromRef;
use serde::Serialize;
use tokio::sync::broadcast;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::core::app_state::AppState;

pub struct VideoState {
  pub channel_tx: broadcast::Sender<VideoStateMessage>,
  pub channel_rx: broadcast::Receiver<VideoStateMessage>,
}

impl Default for VideoState {
  fn default() -> Self {
    let (tx, rx) = broadcast::channel::<VideoStateMessage>(1000);

    Self {
      channel_tx: tx,
      channel_rx: rx,
    }
  }
}

impl FromRef<AppState> for Arc<VideoState> {
  fn from_ref(app_state: &AppState) -> Self {
    app_state.video_state.clone()
  }
}

#[derive(Clone, Debug)]
pub struct VideoStateMessage {
  pub id: Uuid,
  pub message: VideoStateProgress,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct VideoStateProgress {
  pub value: f64,
  pub done: bool,
}
