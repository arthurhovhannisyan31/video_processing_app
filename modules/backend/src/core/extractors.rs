use std::str::FromStr;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::core::error::{InputError, ServerError};

pub const X_USER_ID_HEADER: &str = "x-user-id";

pub struct UserIdExtractor(pub Uuid);

impl<S> FromRequestParts<S> for UserIdExtractor
where
  S: Send + Sync,
{
  type Rejection = ServerError;

  async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
    if let Some(val) = parts.headers.get(X_USER_ID_HEADER) {
      if let Some(user_id) = val.to_str().ok().and_then(|val| Uuid::from_str(val).ok()) {
        Ok(UserIdExtractor(user_id))
      } else {
        Err(InputError::DataError(
          "`X-USER-ID` header has wrong value".to_string(),
        ))?
      }
    } else {
      Err(InputError::DataError(
        "`X-USER-ID` header is missing".to_string(),
      ))?
    }
  }
}
