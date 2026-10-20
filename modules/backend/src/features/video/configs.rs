use std::str::FromStr;

use crate::core::error::{InputError, ServerError};

#[derive(Debug, PartialEq)]
pub enum FieldName {
  Video,
  Operation,
  Codec,
  // preset depends on codec field and should be listed in request payload after codec
  Preset,
  // quality depends on codec field and should be listed in request payload after codec
  Quality,
  Format,
}

impl FromStr for FieldName {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      field_name::CODEC => Ok(FieldName::Codec),
      field_name::FORMAT => Ok(FieldName::Format),
      field_name::OPERATION => Ok(FieldName::Operation),
      field_name::VIDEO => Ok(FieldName::Video),
      field_name::PRESET => Ok(FieldName::Preset),
      field_name::QUALITY => Ok(FieldName::Quality),
      _ => Err(InputError::DataError(format!(
        "Field '{s}' is not supported"
      )))?,
    }
  }
}

pub mod field_name {
  pub const CODEC: &str = "codec";
  pub const FORMAT: &str = "format";
  pub const OPERATION: &str = "operation";
  pub const VIDEO: &str = "video";
  pub const PRESET: &str = "preset";
  pub const QUALITY: &str = "quality";
}
