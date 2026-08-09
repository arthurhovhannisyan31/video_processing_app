use serde::{Deserialize, Serialize};

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
