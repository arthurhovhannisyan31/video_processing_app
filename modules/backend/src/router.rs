use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::core::app_state::AppState;
use crate::core::cors::build_cors_layer;
use crate::core::error::ServerError;
use crate::features::system::routes::get_system_router;
use crate::features::video::routes::get_video_router;

pub mod routes {
  pub const LOGIN: &str = "/auth/login";
  pub const REGISTER: &str = "/auth/register";
  pub const HEALTH: &str = "/health";
  pub const OPENAPI: &str = "/openapi";
  pub const VIDEO_INSPECT: &str = "/video/inspect";
  pub const VIDEO_JOBS: &str = "/video/jobs";
  pub const VIDEO_JOBS_BY_ID: &str = "/video/jobs/{id}";
  pub const VIDEO_JOBS_BY_ID_LOGS: &str = "/video/jobs/{id}/logs";
}

pub fn build_router(app_state: AppState) -> Result<Router, ServerError> {
  let merged_router = Router::new()
    // .merge(get_auth_router(app_state.clone())?)
    .merge(get_system_router())
    .merge(get_video_router(app_state.clone()))
    .with_state(app_state.clone());

  let router = Router::new()
    .nest("/api", merged_router)
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new())
    .layer(build_cors_layer(app_state.app_config.clone()));

  Ok(router)
}
