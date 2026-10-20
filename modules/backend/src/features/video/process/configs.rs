pub const OUTPUT_PATH_SUFFIX: &str = "_output";

pub mod operation {
  pub const COMPRESS: &str = "compress";
}

pub mod codec {
  pub mod x264 {
    use crate::features::video::process::types::{Preset, VideoFormat};

    pub const TYPE: &str = "libx264";

    pub mod quality {
      pub const HIGH: &str = "18";
      pub const STANDARD: &str = "23";
      pub const LOW: &str = "28";
    }

    pub mod preset {
      pub const FAST: &str = "superfast";
      pub const BALANCED: &str = "medium";
      pub const BEST_COMPRESSION: &str = "slow";
    }

    pub fn build_args(preset: &Preset, quality: String, format: &VideoFormat) -> Vec<String> {
      vec![
        "-y".to_string(),
        "-nostdin".to_string(),
        "-loglevel".to_string(),
        "quiet".to_string(),
        "-vcodec".to_string(),
        "libx264".to_string(),
        "-crf".to_string(),
        quality,
        "-preset".to_string(),
        preset.0.to_string(),
        "-acodec".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "128k".to_string(),
        "-progress".to_string(),
        "pipe:2".to_string(),
        "-f".to_string(),
        format.muxer.to_string(),
      ]
    }
  }
  pub mod vp9 {
    use crate::features::video::process::types::VideoFormat;

    pub const TYPE: &str = "libvpx-vp9";

    pub mod quality {
      pub const HIGH: &str = "24";
      pub const STANDARD: &str = "31";
      pub const LOW: &str = "38";
    }

    pub fn build_args(quality: String, format: &VideoFormat) -> Vec<String> {
      vec![
        "-y".to_string(),
        "-nostdin".to_string(),
        "-loglevel".to_string(),
        "quiet".to_string(),
        "-vcodec".to_string(),
        "libvpx-vp9".to_string(),
        "-crf".to_string(),
        quality,
        // Should match container limits
        "-cpu-used".to_string(),
        "4".to_string(),
        "-threads".to_string(),
        "3".to_string(),
        "-tile-columns".to_string(),
        "1".to_string(),
        "-acodec".to_string(),
        "libopus".to_string(),
        "-b:v".to_string(),
        "0".to_string(),
        "-b:a".to_string(),
        "128k".to_string(),
        "-progress".to_string(),
        "pipe:2".to_string(),
        "-f".to_string(),
        format.muxer.to_string(),
      ]
    }
  }
}

pub mod muxer {
  pub const MP4: &str = "mp4";
  pub const MKV: &str = "matroska";
  pub const WEBM: &str = "webm";
  pub const MOV: &str = "mov";
}

pub mod extension {
  pub const MP4: &str = "mp4";
  pub const MKV: &str = "mkv";
  pub const WEBM: &str = "webm";
  pub const MOV: &str = "mov";
}

pub mod mime_type {
  pub const MKV: &str = "video/x-matroska";
  pub const MOV: &str = "video/quicktime";
  pub const MP4: &str = "video/mp4";
  pub const WEBM: &str = "video/webm";
}
