use crate::core::error::ApplicationError;
use crate::features::video::configs::ffprobe::FfprobeType;

use serde_json::Value;

pub fn ffprobe_mapper(
  inspection_data: Value,
) -> Result<FfprobeType, ApplicationError> {
  let data =
    serde_json::from_value::<FfprobeType>(inspection_data).map_err(|err| {
      ApplicationError::Internal(format!(
        "Failed to deserialize ffprobe output: {err}"
      ))
    })?;

  Ok(data)
}
