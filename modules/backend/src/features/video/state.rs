use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::FromRef;
use parking_lot::RwLock;
use serde::Serialize;
use tokio::sync::mpsc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::core::app_state::AppState;

pub type VideoWsConnectionsMap = RwLock<HashMap<Uuid, mpsc::Sender<VideoStateMessage>>>;

pub struct VideoState {
  pub connections_map: VideoWsConnectionsMap,
}

impl Default for VideoState {
  fn default() -> Self {
    Self {
      connections_map: RwLock::new(HashMap::new()),
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
