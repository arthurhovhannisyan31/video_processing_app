use std::sync::Arc;

use parking_lot::Mutex;
use sqlx::PgPool;

use crate::core::app_config::AppConfig;
use crate::core::jwt::JwtService;
use crate::features::auth::repository::PostgresUserRepository;
use crate::features::auth::service::AuthService;
use crate::features::auth::state::AuthState;
use crate::features::system::state::SystemState;
use crate::features::video::service::VideoService;
use crate::features::video::state::VideoState;

#[derive(Clone)]
pub struct AppState {
  pub auth_state: Arc<AuthState>,
  pub app_config: Arc<AppConfig>,
  pub video_state: Arc<VideoState>,
  pub system_state: Arc<SystemState>,
}

impl AppState {
  pub fn new(app_config: AppConfig, pool: PgPool) -> Self {
    let jwt_service = JwtService::new(app_config.jwt_secret.clone());
    let users_repo = PostgresUserRepository::new(pool.clone());
    let auth_service = AuthService::new(users_repo, jwt_service.clone());
    Self {
      auth_state: Arc::new(AuthState {
        auth_service,
        jwt_service,
      }),
      app_config: Arc::new(app_config),
      video_state: Arc::from(VideoState {
        video_service: VideoService::default(),
      }),
      system_state: Arc::new(SystemState {
        db_pool: pool,
        rate_limiters: Arc::new(Mutex::new(vec![])),
      }),
    }
  }
}
