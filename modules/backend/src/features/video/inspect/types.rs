use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::core::error::ServerError;
use crate::features::video::inspect::utils::get_r_frame_rate_from_string;
use crate::features::video::model::{AudioStream, MediaMetadata, VideoStream};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FfprobeOutput {
  pub format: Option<FfprobeFormat>,
  pub streams: Option<Vec<FfprobeStream>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FfprobeFormat {
  #[serde(deserialize_with = "crate::features::video::helpers::deserialize_string_to_type")]
  pub bit_rate: i64,
  #[serde(deserialize_with = "crate::features::video::helpers::deserialize_string_to_type")]
  pub duration: f32,
  pub filename: String,
  pub format_name: String,
  #[serde(deserialize_with = "crate::features::video::helpers::deserialize_string_to_type")]
  pub size: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FfprobeStream {
  pub id: String,
  #[serde(deserialize_with = "crate::features::video::helpers::deserialize_string_to_type")]
  pub bit_rate: i32,
  pub codec_type: String,
  pub codec_long_name: String,
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub r_frame_rate: String,
}

#[derive(PartialEq)]
pub enum CodecType {
  Audio,
  Video,
}

impl FromStr for CodecType {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "audio" => Ok(CodecType::Audio),
      "video" => Ok(CodecType::Video),
      _ => Err(ServerError::DataError(format!(
        "Codec type is not supported: {s}"
      ))),
    }
  }
}

impl TryFrom<FfprobeOutput> for MediaMetadata {
  type Error = ServerError;

  fn try_from(value: FfprobeOutput) -> Result<Self, Self::Error> {
    let FfprobeOutput { format, streams } = value;

    let format = format.ok_or(ServerError::MissingMediaData(
      "Missing format from ffprobe output".to_string(),
    ))?;
    let streams = streams.ok_or(ServerError::MissingMediaData(
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
      let codec_type = CodecType::from_str(stream.codec_type.as_str())?;

      match codec_type {
        CodecType::Audio => {
          media_metadata.audio_streams.push(AudioStream {
            id: stream.id,
            bit_rate: stream.bit_rate,
            codec: stream.codec_long_name,
          });
        }
        CodecType::Video => media_metadata.video_streams.push(VideoStream {
          id: stream.id,
          bit_rate: stream.bit_rate,
          codec: stream.codec_long_name,
          width: stream.width.unwrap_or_default(),
          height: stream.height.unwrap_or_default(),
          fps: get_r_frame_rate_from_string(stream.r_frame_rate)?,
        }),
      }
    }

    Ok(media_metadata)
  }
}

#[derive(Debug, Default)]
pub struct InspectVideoMeta {
  pub file_name: String,
  pub local_path: String,
}
