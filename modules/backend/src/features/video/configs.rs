pub const DEFAULT_VIDEO_BODY_LIMIT_BYTES: usize = 20 * 1024 * 1024;

pub mod ffprobe {
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Serialize, Deserialize)]
  pub struct FfprobeType {
    pub(crate) format: Option<FormatType>,
    // streams: Option<Vec<StreamType>>,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct FormatType {
    pub filename: String,
    #[serde(
      deserialize_with = "crate::features::video::helpers::deserialize_string_to_i64"
    )]
    pub size: i64,
    #[serde(
      deserialize_with = "crate::features::video::helpers::deserialize_string_to_f32"
    )]
    pub duration: f32,
    pub format_name: String,
    // // ?
    // pub nb_streams: i32,
    // pub nb_programs: i32,
    // pub nb_stream_groups: i32,
    // pub format_long_name: String,
    // pub start_time: f32,
    // pub bit_rate: f64,
    // pub probe_score: i32,
  }

  pub struct StreamType {
    pub index: i32,
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: String,
    pub codec_type: String,
    pub codec_tag: String,
    pub codec_tag_string: String,
    pub extradata: String,
    pub extradata_size: i32,
    pub extradata_hash: String,
    pub mime_codec_string: String,
    pub id: String,
    pub r_frame_rate: String,
    pub avg_frame_rate: String,
    pub time_base: String,
    pub start_pts: f64,
    pub start_time: f32,
    pub duration_ts: f64,
    pub duration: f32,
    pub bit_rate: i32,
    pub max_bit_rate: i32,
    pub bits_per_raw_sample: i32,
    pub nb_frames: i32,
    pub nb_read_frames: i32,
    pub nb_read_packets: i32,
    // video attributes
    pub width: i32,
    pub height: i32,
    pub coded_width: i32,
    pub coded_height: i32,
    pub closed_captions: bool,
    pub film_grain: bool,
    pub has_b_frames: i32,
    pub sample_aspect_ratio: String,
    pub display_aspect_ratio: String,
    pub pix_fmt: String,
    pub level: i32,
    pub color_range: String,
    pub color_space: String,
    pub color_transfer: String,
    pub color_primaries: String,
    pub chroma_location: String,
    pub field_order: String,
    pub refs: i32,
    // audio attributes
    pub sample_fmt: String,
    pub sample_rate: i32,
    pub channels: i32,
    pub channel_layout: String,
    pub bits_per_sample: i32,
    pub initial_padding: i32,
  }
}

// TOOD move to moduel FfprobeTypes

// file path
// file size
// duration
// container format
// video streams
// audio streams
// width / height
// frame rate
// video codec
// audio codec
// bitrate
// aspect ratio
