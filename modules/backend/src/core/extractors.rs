use std::str::FromStr;

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use serde_json::json;
use uuid::Uuid;

pub const X_USER_ID_HEADER: &str = "x-user-id";

pub struct XUserIdExtractor(pub Uuid);

impl<S> FromRequestParts<S> for XUserIdExtractor
where
  S: Send + Sync,
{
  type Rejection = (StatusCode, String);

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    if let Some(val) = parts.headers.get(X_USER_ID_HEADER) {
      if let Some(user_id) = val.to_str().ok().and_then(|val| Uuid::from_str(val).ok()) {
        Ok(XUserIdExtractor(user_id))
      } else {
        Err((
          StatusCode::BAD_REQUEST,
          json!({"message": "`X-USER-ID` header has wrong value"}).to_string(),
        ))
      }
    } else {
      Err((
        StatusCode::BAD_REQUEST,
        json!({"message": "`X-USER-ID` header is missing"}).to_string(),
      ))
    }
  }
}
