use crate::core::error::ApplicationError;

pub const OUTPUT_PATH_SUFFIX: &str = "_output";

pub mod operation {
  pub const COMPRESS: &str = "compress";
}

enum FieldName {
  Video,
  Operation,
}

pub mod preset {
  pub fn compress<'a>() -> Vec<&'a str> {
    vec![
      "-y",
      "-nostdin",
      "-vcodec",
      "libx264",
      "-crf",
      "23",
      "-preset",
      "medium",
      "-acodec",
      "aac",
      "-b:a",
      "128k",
      "-progress",
      "pipe:2",
    ]
  }
}

pub fn get_preset_by_name<'a>(operation: &str) -> Result<Vec<&'a str>, ApplicationError> {
  match operation {
    operation::COMPRESS => Ok(preset::compress()),
    _ => Err(ApplicationError::BadRequest(format!(
      "Unsupported operation type: {operation}"
    ))),
  }
}
