use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::FromRef;
use mini_moka::sync::Cache;
use parking_lot::RwLock;
use serde::Serialize;
use tokio::sync::mpsc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::core::app_state::AppState;
use crate::features::video::cache::build_cache;
use crate::features::video::model::MediaMetadata;

pub type VideoWsConnectionsMap = RwLock<HashMap<Uuid, mpsc::Sender<VideoStateProgress>>>;

pub struct VideoState {
  pub connections_map: VideoWsConnectionsMap,
  pub cache: Cache<String, MediaMetadata>,
}

impl Default for VideoState {
  fn default() -> Self {
    Self {
      connections_map: RwLock::new(HashMap::new()),
      cache: build_cache(),
    }
  }
}

impl FromRef<AppState> for Arc<VideoState> {
  fn from_ref(app_state: &AppState) -> Self {
    app_state.video_state.clone()
  }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct VideoStateProgress {
  pub file_name: String,
  pub value: f64,
  pub done: bool,
}
