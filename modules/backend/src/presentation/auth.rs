use crate::application::error::ApplicationError;
use crate::domain::user::User;
use crate::presentation::constants::routes;
use crate::presentation::{
  dto::{AuthRequest, AuthResponse, AuthenticatedUser, CreateUserRequest},
  state::AppState,
  state::AuthState,
};

use axum::{
  Json, Router,
  body::Body,
  extract::State,
  http::{Response, StatusCode},
  response::IntoResponse,
  routing::post,
};
use serde_json::json;
use std::sync::Arc;
use tracing::info;
use validator::Validate;

pub fn get_auth_router() -> Router<AppState> {
  Router::new()
    .route(routes::LOGIN, post(login))
    .route(routes::REGISTER, post(register))
}

#[utoipa::path(
  post,
  path = routes::LOGIN,
  responses((status = OK, body = AuthResponse))
)]
pub async fn login(
  State(auth_state): State<Arc<AuthState>>,
  Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
  let token = auth_state
    .auth_service
    .login(&payload.email, &payload.password)
    .await?;
  let user = auth_state.auth_service.get_by_email(&payload.email).await?;

  build_auth_response(StatusCode::OK, token.clone(), user)
}

#[utoipa::path(
  post,
  path = routes::REGISTER,
  responses((status = OK, body = AuthResponse))
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
    .generate_token(user.id.clone(), user.username.clone())
    .map_err(|err| ApplicationError::Internal(err.to_string()))?;

  build_auth_response(StatusCode::CREATED, token.clone(), user)
}

fn build_auth_response(
  status: StatusCode,
  token: String,
  user: User,
) -> Result<impl IntoResponse, ApplicationError> {
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

  Response::builder()
    .status(status)
    .body(Body::from(response_body))
    .map_err(|err| ApplicationError::Internal(err.to_string()))
}
