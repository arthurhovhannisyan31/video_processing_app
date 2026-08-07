use crate::core::error::{ApplicationError, ServerError};
use anyhow::anyhow;

pub fn get_r_frame_rate_from_string(
  input: String,
) -> Result<f32, ApplicationError> {
  if let Some((val1, val2)) = input.split_once("/") {
    let val1 = val1.parse::<f32>().map_err(ServerError::ParseFloatError)?;
    let val2 = val2.parse::<f32>().map_err(ServerError::ParseFloatError)?;

    return Ok(val1 / val2);
  }

  Err(ServerError::OtherError(anyhow!(
    "Failed to parse r_frame_rate from string"
  )))?
}
