use std::sync::Arc;

use axum::extract::FromRef;

use crate::core::app_state::AppState;
use crate::features::video::process::service::VideoService;

pub struct VideoState {
  pub video_service: VideoService,
}

impl FromRef<AppState> for Arc<VideoState> {
  fn from_ref(app_state: &AppState) -> Self {
    app_state.video_state.clone()
  }
}
