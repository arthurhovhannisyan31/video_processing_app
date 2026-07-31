use crate::core::error::ApplicationError;

use serde_json::{Value, json};

pub async fn ffprobe_mapper() -> Result<Value, ApplicationError> {
  //

  Ok(json!("{}"))
}
