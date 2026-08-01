use serde::Serialize;

#[derive(Serialize)]
pub struct VideoInspectionResponse {
  pub original_file_name: String,
  pub file_size_bytes: usize,
  pub duration_seconds: f32,
  pub format_name: String,

  pub video_streams: Vec<String>,
  pub width: usize,
  pub height: usize,
  pub fps: usize,
  pub codecs: Vec<String>,
  pub bitrate: usize,

  pub audio_streams: Vec<String>,
  pub audio_stream_count: usize,
}
