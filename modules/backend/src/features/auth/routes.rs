use std::sync::Arc;

use anyhow::anyhow;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::json;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tracing::info;
use validator::Validate;

use crate::core::app_state::{AppState, AuthState};
use crate::core::error::{ApplicationError, ServerError};
use crate::features::auth::dto::{AuthRequest, AuthResponse, AuthenticatedUser, CreateUserRequest};
use crate::features::auth::model::User;
use crate::router::routes;

pub fn get_auth_router() -> Result<Router<AppState>, ServerError> {
  let auth_limiter = GovernorConfigBuilder::default()
    .key_extractor(SmartIpKeyExtractor)
    .finish()
    .ok_or(ServerError::OtherError(anyhow!(
      "Wrong tower_governor configuration"
    )))?;

  let router = Router::new()
    .route(routes::LOGIN, post(login))
    .route(routes::REGISTER, post(register))
    .layer(GovernorLayer::new(auth_limiter));

  Ok(router)
}

#[utoipa::path(
  post,
  path = routes::LOGIN,
  request_body = AuthRequest,
  responses(
    (status = OK,description = "Success", body = AuthResponse),
    (status = UNAUTHORIZED, description = "User not found or credentials are invalid"),
    (status = INTERNAL_SERVER_ERROR, description = "Server internal error", body = Object, content_type = "application/json")
  )
)]
pub async fn login(
  State(auth_state): State<Arc<AuthState>>,
  Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
  payload.validate()?;

  let token = auth_state
    .auth_service
    .login(&payload.email, &payload.password)
    .await?;
  let user = auth_state.auth_service.get_by_email(&payload.email).await?;
  Ok(build_auth_response(StatusCode::OK, token.clone(), user)?)
}

#[utoipa::path(
  post,
  path = routes::REGISTER,
  request_body = CreateUserRequest,
  responses(
    (status = OK,description = "Success", body = AuthResponse),
    (status = BAD_REQUEST, description = "Credentials requirements are not met", body = Object,content_type = "application/json"),
    (status = INTERNAL_SERVER_ERROR, description = "Server internal error", body = Object, content_type = "application/json")
  )
)]
async fn register(
  State(auth_state): State<Arc<AuthState>>,
  Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
  payload.validate()?;

  let user = auth_state
    .auth_service
    .register(
      payload.email.clone(),
      payload.password.clone(),
      payload.username.clone(),
    )
    .await?;
  info!(user_id = %user.id, email = %user.email, username = %user.username, "user registered");

  let token = auth_state
    .jwt_service
    .generate_token(user.id.clone(), user.username.clone())?;

  Ok(build_auth_response(
    StatusCode::CREATED,
    token.clone(),
    user,
  )?)
}

fn build_auth_response(
  status: StatusCode,
  token: String,
  user: User,
) -> Result<impl IntoResponse, ServerError> {
  let authenticated_user = AuthenticatedUser {
    user_id: user.id.to_string(),
    email: user.email,
    username: user.username,
  };
  let response_body = json!(AuthResponse {
    user: authenticated_user,
    token: token.clone(),
  })
  .to_string();

  Ok(
    Response::builder()
      .status(status)
      .body(Body::from(response_body))
      .map_err(|err| ApplicationError::Internal(err.to_string())),
  )
}
