use std::str::FromStr;

use crate::core::error::{InputError, ServerError};

#[derive(Debug, PartialEq)]
pub enum FieldName {
  Video,
  Operation,
  Codec,
  // preset depends on codec field and should be listed in request payload after codec
  Preset,
  Quality,
}

impl FromStr for FieldName {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "video" => Ok(FieldName::Video),
      "operation" => Ok(FieldName::Operation),
      "codec" => Ok(FieldName::Codec),
      "preset" => Ok(FieldName::Preset),
      "quality" => Ok(FieldName::Quality),
      _ => Err(InputError::DataError(format!(
        "Field name is not supported: {s}"
      )))?,
    }
  }
}
