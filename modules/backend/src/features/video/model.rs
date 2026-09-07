use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct MediaMetadata {
  pub path: String,
  pub duration_seconds: f64,
  pub file_size_bytes: i64,
  pub video_streams: Vec<VideoStream>,
  pub audio_streams: Vec<AudioStream>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoStream {
  pub id: String,
  pub bit_rate: i32,
  pub codec: String,
  pub fps: f32,
  pub height: i32,
  pub width: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioStream {
  pub id: String,
  pub bit_rate: i32,
  pub codec: String,
}
