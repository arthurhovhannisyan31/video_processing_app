use std::str::FromStr;

use crate::core::error::ServerError;

pub const OUTPUT_PATH_SUFFIX: &str = "_output";

#[derive(PartialEq)]
pub enum OperationType {
  Compress,
}

impl FromStr for OperationType {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "compress" => Ok(OperationType::Compress),
      _ => Err(ServerError::DataError(format!(
        "Form operation type is not supported: {s}"
      ))),
    }
  }
}

#[derive(PartialEq)]
pub enum FieldName {
  Video,
  Operation,
}

impl FromStr for FieldName {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "video" => Ok(FieldName::Video),
      "operation" => Ok(FieldName::Operation),
      _ => Err(ServerError::DataError(format!(
        "Field name is not supported: {s}"
      ))),
    }
  }
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

pub fn get_preset_by_name<'a>(operation: &str) -> Result<Vec<&'a str>, ServerError> {
  let operation_type = OperationType::from_str(operation)?;

  match operation_type {
    OperationType::Compress => Ok(preset::compress()),
  }
}
