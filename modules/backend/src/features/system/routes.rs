use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::response::Json;
use axum::routing::get;
use chrono::Utc;
use serde_json::{Value, json};
use utoipa::OpenApi;

use crate::core::app_state::AppState;
use crate::core::error::ApplicationError;
use crate::core::openapi::OpenApiSpec;
use crate::features::system::state::SystemState;
use crate::router::routes;

pub fn get_system_router() -> Router<AppState> {
  Router::new()
    .route(routes::HEALTH, get(health))
    .route(routes::READY, get(ready))
    .route(routes::OPENAPI, get(openapi))
}

#[utoipa::path(
  get,
  path = routes::READY,
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
  path = routes::READY,
  responses((status = OK, body = Value))
)]
async fn ready(State(system_state): State<Arc<SystemState>>) -> Result<(), ApplicationError> {
  system_state
    .db_pool
    .acquire()
    .await
    .map_err(|err| ApplicationError::ServiceUnavailable(err.to_string()))?;

  Ok(())
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
