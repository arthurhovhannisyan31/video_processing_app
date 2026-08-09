use std::sync::Arc;

use axum::http::{HeaderValue, Method, header};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::core::app_config::AppConfig;

pub fn build_cors_layer(app_config: Arc<AppConfig>) -> CorsLayer {
  let origin_values: Vec<HeaderValue> = app_config
    .cors_origins
    .iter()
    .filter_map(|el| el.parse().ok())
    .collect();

  CorsLayer::new()
    .allow_origin(AllowOrigin::list(origin_values))
    .allow_methods([
      Method::OPTIONS,
      Method::GET,
      Method::POST,
      Method::PUT,
      Method::DELETE,
    ])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
    .allow_credentials(true)
}
