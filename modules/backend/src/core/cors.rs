use std::sync::Arc;

use axum::http::{HeaderValue, Method, header};
use headers::HeaderName;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::log::warn;

use crate::core::app_config::AppConfig;
use crate::core::extractors::X_USER_ID_HEADER;

pub fn build_cors_layer(app_config: Arc<AppConfig>) -> CorsLayer {
  let origin_values: Vec<HeaderValue> = app_config
    .cors_origins
    .iter()
    .filter_map(|el| {
      el.parse()
        .inspect_err(|err| warn!("Invalid cors origin {el}: {err}"))
        .ok()
    })
    .collect();

  let x_user_id_header = HeaderName::from_static(X_USER_ID_HEADER);

  CorsLayer::new()
    .allow_origin(AllowOrigin::list(origin_values))
    .allow_methods([
      Method::OPTIONS,
      Method::GET,
      Method::POST,
      Method::PUT,
      Method::DELETE,
    ])
    .allow_headers([
      header::AUTHORIZATION,
      header::CONTENT_TYPE,
      header::ACCEPT,
      x_user_id_header,
    ])
    .allow_credentials(true)
}
