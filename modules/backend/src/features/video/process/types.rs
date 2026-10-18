use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::core::error::{InputError, ServerError};
use crate::features::video::process::configs::{codec, operation, preset, quality};

#[derive(Debug, Default)]
pub struct ProcessVideoMeta {
  pub file_name: String,
  pub local_path: String,
  pub operation: Operation,
  pub codec: Codec,
  // preset depends on codec field and should be listed in request payload after codec
  pub preset: Preset,
  pub quality: Quality,
}

#[derive(Debug, Default, PartialEq)]
pub enum Operation {
  #[default]
  Compress,
}

#[derive(Debug, Default, PartialEq)]
pub enum Codec {
  #[default]
  X264,
}

#[derive(Debug, Default, PartialEq)]
pub enum X264Preset {
  #[default]
  Fast,
  Medium,
  Slow,
}

#[derive(Debug, Default, PartialEq)]
pub enum Quality {
  #[default]
  Standard,
  High,
  Low,
}

impl FromStr for Operation {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      operation::COMPRESS => Ok(Operation::Compress),
      "" => Err(InputError::DataError("Operation is missing".to_string()))?,
      _ => Err(InputError::DataError(format!(
        "Operation is not supported: {s}"
      )))?,
    }
  }
}

impl FromStr for Codec {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      codec::X264 => Ok(Codec::X264),
      "" => Err(InputError::DataError("Codec is missing".to_string()))?,
      _ => Err(InputError::DataError(format!(
        "Codec is not supported: {s}"
      )))?,
    }
  }
}

impl Display for Codec {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::X264 => write!(f, "{}", codec::X264),
    }
  }
}

#[derive(Debug, Default, PartialEq)]
pub struct Preset(pub(crate) String);

impl Preset {
  pub fn get_by_codec(s: &str, codec: &Codec) -> Result<Self, ServerError> {
    match codec {
      Codec::X264 => X264Preset::from_str(s).map(|val| Self(val.to_string())),
    }
  }
}

impl FromStr for X264Preset {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      preset::x_264::BALANCED => Ok(X264Preset::Medium),
      preset::x_264::BEST_COMPRESSION => Ok(X264Preset::Slow),
      preset::x_264::FAST => Ok(X264Preset::Fast),
      "" => Err(InputError::DataError("Preset is missing".to_string()))?,
      _ => Err(InputError::DataError(format!(
        "Preset is not supported: {s}"
      )))?,
    }
  }
}

impl Display for X264Preset {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Fast => write!(f, "{}", preset::x_264::FAST),
      Self::Medium => write!(f, "{}", preset::x_264::BALANCED),
      Self::Slow => write!(f, "{}", preset::x_264::BEST_COMPRESSION),
    }
  }
}

impl FromStr for Quality {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      quality::STANDARD => Ok(Quality::Standard),
      quality::HIGH => Ok(Quality::High),
      quality::LOW => Ok(Quality::Low),
      "" => Err(InputError::DataError("Preset is missing".to_string()))?,
      _ => Err(InputError::DataError(format!(
        "Preset is not supported: {s}"
      )))?,
    }
  }
}

impl Display for Quality {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Standard => write!(f, "{}", quality::STANDARD),
      Self::High => write!(f, "{}", quality::HIGH),
      Self::Low => write!(f, "{}", quality::LOW),
    }
  }
}
