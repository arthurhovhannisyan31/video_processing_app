use serde_json::Value;

use crate::core::error::ServerError;
use crate::features::video::inspect::types::FfprobeOutput;
use crate::features::video::model::MediaMetadata;

pub fn map_media_meta(inspection_data: Value) -> Result<MediaMetadata, ServerError> {
  let data = serde_json::from_value::<FfprobeOutput>(inspection_data).map_err(|err| {
    ServerError::DataError(format!("Failed to deserialize 'ffprobe' output: {err}"))
  })?;

  MediaMetadata::try_from(data)
}
