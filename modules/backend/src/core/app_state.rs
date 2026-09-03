use std::sync::Arc;

use crate::core::app_config::AppConfig;
use crate::features::auth::state::AuthState;
use crate::features::video::state::VideoState;

#[derive(Clone)]
pub struct AppState {
  pub auth_state: Arc<AuthState>,
  pub app_config: Arc<AppConfig>,
  pub video_state: Arc<VideoState>,
}
