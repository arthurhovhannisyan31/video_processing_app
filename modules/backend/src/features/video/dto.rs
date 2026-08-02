use crate::features::video::configs::ffprobe::FfprobeType;
use std::path::Path;

use serde::Serialize;

#[derive(Serialize)]
pub struct VideoInspectionResponse {
  pub original_file_name: String,
  pub file_size_bytes: i64,
  pub duration_seconds: f32,
  pub format_name: String,
  pub video_streams: Vec<String>,
  pub width: i32,
  pub height: i32,
  pub fps: String,
  pub codecs: Vec<String>,
  pub bitrate: i64,
  pub audio_streams: Vec<String>,
  pub audio_stream_count: usize,
}

impl From<FfprobeType> for VideoInspectionResponse {
  fn from(data: FfprobeType) -> Self {
    let FfprobeType { format, streams } = data;
    let format = format.unwrap_or_default();
    let streams = streams.unwrap_or_default();
    let original_file_name = Path::new(&format.filename)
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string();

    println!("streams: {streams:#?}");

    let mut response = Self {
      original_file_name,
      file_size_bytes: format.size,
      duration_seconds: format.duration,
      format_name: format.format_name,
      video_streams: vec![],
      width: 0,
      height: 0,
      fps: "".into(),
      codecs: vec![],
      bitrate: format.bit_rate,
      audio_streams: vec![],
      audio_stream_count: 0,
    };

    for stream in streams {
      match stream.codec_type.as_str() {
        "audio" => {
          response.audio_streams.push(stream.id);
          response.codecs.push(stream.codec_long_name);
          response.audio_stream_count += 1;
        }
        "video" => {
          response.video_streams.push(stream.id);
          response.codecs.push(stream.codec_long_name);
          response.width = stream.width.unwrap_or_default();
          response.height = stream.height.unwrap_or_default();
          response.fps = stream.r_frame_rate;
        }
        _ => {}
      }
    }

    response
  }
}
