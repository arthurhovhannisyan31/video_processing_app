use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::features::video::model::{AudioStream, MediaMetadata, VideoStream};

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInspectionResponse {
  pub original_file_name: String,
  pub file_size_bytes: i64,
  pub duration_seconds: f32,
  pub video_streams: Vec<VideoStream>,
  pub audio_streams: Vec<AudioStream>,
}

impl From<MediaMetadata> for VideoInspectionResponse {
  fn from(data: MediaMetadata) -> Self {
    let MediaMetadata {
      path,
      duration_seconds,
      file_size_bytes,
      video_streams,
      audio_streams,
    } = data;
    let original_file_name = Path::new(&path)
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string();

    Self {
      original_file_name,
      file_size_bytes,
      duration_seconds,
      video_streams,
      audio_streams,
    }
  }
}
