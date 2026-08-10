use serde::{Deserialize, Serialize};

use crate::core::error::{ApplicationError, DomainError};
use crate::features::video::inspect::types::FfprobeOutput;
use crate::features::video::inspect::utils::get_r_frame_rate_from_string;

#[derive(Debug, Serialize)]
pub struct MediaMetadata {
  pub path: String,
  pub duration_seconds: f32,
  pub file_size_bytes: i64,
  pub video_streams: Vec<VideoStream>,
  pub audio_streams: Vec<AudioStream>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoStream {
  pub id: String,
  pub bit_rate: i32,
  pub codec: String,
  pub fps: f32,
  pub height: i32,
  pub width: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioStream {
  pub id: String,
  pub bit_rate: i32,
  pub codec: String,
}

impl TryFrom<FfprobeOutput> for MediaMetadata {
  type Error = ApplicationError;

  fn try_from(value: FfprobeOutput) -> Result<Self, Self::Error> {
    let FfprobeOutput { format, streams } = value;

    let format = format.ok_or(DomainError::MissingMediaData(
      "Missing format from ffprobe output".to_string(),
    ))?;
    let streams = streams.ok_or(DomainError::MissingMediaData(
      "Missing format from ffprobe output".to_string(),
    ))?;

    let mut media_metadata = Self {
      path: format.filename,
      duration_seconds: format.duration,
      file_size_bytes: format.size,
      video_streams: vec![],
      audio_streams: vec![],
    };

    for stream in streams {
      match stream.codec_type.as_str() {
        "audio" => {
          media_metadata.audio_streams.push(AudioStream {
            id: stream.id,
            bit_rate: stream.bit_rate,
            codec: stream.codec_long_name,
          });
        }
        "video" => media_metadata.video_streams.push(VideoStream {
          id: stream.id,
          bit_rate: stream.bit_rate,
          codec: stream.codec_long_name,
          width: stream.width.unwrap_or_default(),
          height: stream.height.unwrap_or_default(),
          fps: get_r_frame_rate_from_string(stream.r_frame_rate)?,
        }),
        _ => {}
      }
    }

    Ok(media_metadata)
  }
}
