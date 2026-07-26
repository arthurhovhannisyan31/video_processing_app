use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::core::app_state::AppState;
use crate::core::cors::build_cors_layer;
use crate::features::auth::routes::get_auth_router;
use crate::features::protected::routes::get_protected_router;
use crate::features::system::routes::get_utilities_router;

pub mod routes {
  pub const LOGIN: &str = "/auth/login";
  pub const REGISTER: &str = "/auth/register";
  pub const HEALTH: &str = "/health";
  pub const OPENAPI: &str = "/openapi";
  pub const PROTECTED: &str = "/protected";
}

pub fn build_router(app_state: AppState) -> Router {
  let merged_router = Router::new()
    .merge(get_auth_router())
    .merge(get_utilities_router())
    .merge(get_protected_router(app_state.clone()))
    .with_state(app_state.clone());

  Router::new()
    .nest("/api", merged_router)
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new())
    .layer(build_cors_layer(app_state.app_config.clone()))
}
