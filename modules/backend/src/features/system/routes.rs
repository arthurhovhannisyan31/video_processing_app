use crate::core::app_state::AppState;
use crate::core::error::ApplicationError;
use crate::core::openapi::OpenApiSpec;

use crate::router::routes;
use axum::{Router, response::Json, routing::get};
use chrono::Utc;
use serde_json::{Value, json};
use utoipa::OpenApi;

pub fn get_utilities_router() -> Router<AppState> {
  Router::new()
    .route(routes::HEALTH, get(health))
    .route(routes::OPENAPI, get(openapi))
}

#[utoipa::path(
  get,
  path = routes::HEALTH,
  responses((status = OK, body = Value))
)]
async fn health() -> Json<Value> {
  Json(json!({
    "status": "ok",
    "timestamp": Utc::now(),
  }))
}

#[utoipa::path(
  get,
  path = routes::OPENAPI,
  responses((status = OK, body = Value))
)]
async fn openapi() -> Result<String, ApplicationError> {
  match OpenApiSpec::openapi().to_json() {
    Ok(res) => Ok(res),
    Err(err) => Err(ApplicationError::Internal(err.to_string())),
  }
}
