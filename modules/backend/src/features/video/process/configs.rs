pub const OUTPUT_PATH_SUFFIX: &str = "_output";

pub mod preset {
  use crate::features::video::process::types::{Codec, Preset, Quality};

  pub fn build_compress_args<'a>(codec: Codec, preset: Preset, quality: Quality) -> Vec<String> {
    vec![
      "-y".to_string(),
      "-nostdin".to_string(),
      "-loglevel".to_string(),
      "quiet".to_string(),
      "-vcodec".to_string(),
      // "libx264",
      codec.to_string(),
      "-crf".to_string(),
      // "23",
      quality.to_string(),
      "-preset".to_string(),
      // "medium",
      preset.0.to_string(),
      "-acodec".to_string(),
      "aac".to_string(),
      "-b:a".to_string(),
      "128k".to_string(),
      "-progress".to_string(),
      "pipe:2".to_string(),
    ]
  }

  pub mod x_264 {
    pub const FAST: &str = "superfast";
    pub const BALANCED: &str = "medium";
    pub const BEST_COMPRESSION: &str = "slow";
  }
}

pub mod operation {
  pub const COMPRESS: &str = "compress";
}

pub mod codec {
  pub const X264: &str = "libx264";
}

pub mod quality {
  pub const HIGH: &str = "18";
  pub const STANDARD: &str = "23";
  pub const LOW: &str = "28";
}
