pub const DEFAULT_VIDEO_BODY_LIMIT_BYTES: usize = 20 * 1024 * 1024;

pub mod ffprobe {
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Serialize, Deserialize, Default)]
  pub struct FfprobeType {
    pub format: Option<FormatType>,
    pub streams: Option<Vec<StreamType>>,
  }

  #[derive(Debug, Serialize, Deserialize, Default)]
  pub struct FormatType {
    pub filename: String,
    #[serde(
      deserialize_with = "crate::features::video::helpers::deserialize_string_to_type"
    )]
    pub size: i64,
    #[serde(
      deserialize_with = "crate::features::video::helpers::deserialize_string_to_type"
    )]
    pub duration: f32,
    pub format_name: String,
    #[serde(
      deserialize_with = "crate::features::video::helpers::deserialize_string_to_type"
    )]
    pub bit_rate: i64,
  }

  #[derive(Debug, Serialize, Deserialize, Default)]
  pub struct StreamType {
    pub id: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub r_frame_rate: String,
    pub codec_long_name: String,
    #[serde(
      deserialize_with = "crate::features::video::helpers::deserialize_string_to_type"
    )]
    pub bit_rate: i32,
    pub codec_type: String,
  }
}
