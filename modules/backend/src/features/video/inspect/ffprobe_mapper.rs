use serde_json::Value;

use crate::core::error::ApplicationError;
use crate::features::video::inspect::types::FfprobeOutput;
use crate::features::video::model::MediaMetadata;

pub fn ffprobe_mapper(inspection_data: Value) -> Result<MediaMetadata, ApplicationError> {
  let data = serde_json::from_value::<FfprobeOutput>(inspection_data).map_err(|err| {
    ApplicationError::Internal(format!("Failed to deserialize 'ffprobe' output: {err}"))
  })?;

  Ok(MediaMetadata::try_from(data)?)
}
