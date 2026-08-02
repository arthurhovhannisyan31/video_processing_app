use crate::core::error::ApplicationError;
use crate::features::video::configs::ffprobe::FfprobeType;
use crate::features::video::dto::VideoInspectionResponse;

use serde_json::Value;

pub fn ffprobe_mapper(
  inspection_data: Value,
) -> Result<VideoInspectionResponse, ApplicationError> {
  let data =
    serde_json::from_value::<FfprobeType>(inspection_data).map_err(|err| {
      ApplicationError::Internal(format!(
        "Failed to deserialize ffprobe output: {err}"
      ))
    })?;

  Ok(VideoInspectionResponse::from(data))
}
