use crate::features::video::model::MediaMetadata;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInspectionResponse {
  pub original_file_name: String,
  pub file_size_bytes: i64,
  pub duration_seconds: f32,
  pub width: i32,
  pub height: i32,
  pub fps: f32,
  pub codecs: Vec<String>,
  pub audio_stream_count: usize,
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

    let mut response = Self {
      original_file_name,
      file_size_bytes,
      duration_seconds,
      width: 0,
      height: 0,
      fps: 0.0,
      codecs: vec![],
      audio_stream_count: 0,
    };

    for video_stream in video_streams {
      response.codecs.push(video_stream.codec);
      response.width = video_stream.width;
      response.height = video_stream.height;
      response.fps = video_stream.fps;
    }

    for audio_stream in audio_streams {
      response.codecs.push(audio_stream.codec);
      response.audio_stream_count += 1;
    }

    response
  }
}
