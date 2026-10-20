use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::core::error::{InputError, ServerError};
use crate::features::video::process::configs::{codec, extension, mime_type, muxer, operation};

#[derive(Debug)]
pub struct ProcessVideoMeta {
  pub file_name: String,
  pub local_path: String,
  pub operation: Operation,
  pub codec: Codec,
  // preset depends on codec field and should be listed in request payload after codec
  pub preset: Preset,
  pub quality: Quality,
  pub format: VideoFormat,
}

#[derive(Debug)]
pub struct OutputVideoMeta {
  pub name: String,
  pub path: String,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
  Compress,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Codec {
  X264,
  VP9,
}

#[derive(Debug, PartialEq)]
pub enum X264Preset {
  Fast,
  Medium,
  Slow,
}

#[derive(Debug, PartialEq)]
pub enum Quality {
  Standard,
  High,
  Low,
}

#[derive(Debug, PartialEq)]
pub struct VideoFormat {
  pub extension: Extension,
  pub muxer: String,
  pub mime_type: String,
}

#[derive(Debug, PartialEq)]
pub enum Extension {
  Mp4,
  Mkv,
  Webm,
  Mov,
}

#[derive(Debug, PartialEq)]
pub struct Preset(pub(crate) String);

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
      codec::x264::TYPE => Ok(Codec::X264),
      codec::vp9::TYPE => Ok(Codec::VP9),
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
      Self::X264 => write!(f, "{}", codec::x264::TYPE),
      Self::VP9 => write!(f, "{}", codec::vp9::TYPE),
    }
  }
}

impl Preset {
  pub fn get_by_codec(s: &str, codec: &Codec) -> Result<Self, ServerError> {
    match codec {
      Codec::X264 => X264Preset::from_str(s).map(|val| Self(val.to_string())),
      // Skip value
      _ => Ok(Self("".to_string())),
    }
  }
}

impl FromStr for X264Preset {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      codec::x264::preset::BALANCED => Ok(X264Preset::Medium),
      codec::x264::preset::BEST_COMPRESSION => Ok(X264Preset::Slow),
      codec::x264::preset::FAST => Ok(X264Preset::Fast),
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
      Self::Fast => write!(f, "{}", codec::x264::preset::FAST),
      Self::Medium => write!(f, "{}", codec::x264::preset::BALANCED),
      Self::Slow => write!(f, "{}", codec::x264::preset::BEST_COMPRESSION),
    }
  }
}

impl Quality {
  pub(crate) fn from_str(s: &str, codec: &Codec) -> Result<Self, ServerError> {
    match codec {
      Codec::X264 => match s {
        codec::x264::quality::STANDARD => Ok(Quality::Standard),
        codec::x264::quality::HIGH => Ok(Quality::High),
        codec::x264::quality::LOW => Ok(Quality::Low),
        "" => Err(InputError::DataError("Quality is missing".to_string()))?,
        _ => Err(InputError::DataError(format!(
          "Quality '{s}' is not supported for x264 codec"
        )))?,
      },
      Codec::VP9 => match s {
        codec::vp9::quality::STANDARD => Ok(Quality::Standard),
        codec::vp9::quality::HIGH => Ok(Quality::High),
        codec::vp9::quality::LOW => Ok(Quality::Low),
        "" => Err(InputError::DataError("Quality is missing".to_string()))?,
        _ => Err(InputError::DataError(format!(
          "Quality '{s}' is not supported for vp9 codec"
        )))?,
      },
    }
  }
  pub(crate) fn to_string(&self, codec: &Codec) -> String {
    match codec {
      Codec::X264 => match self {
        Quality::Standard => codec::x264::quality::STANDARD.to_string(),
        Quality::High => codec::x264::quality::HIGH.to_string(),
        Quality::Low => codec::x264::quality::LOW.to_string(),
      },
      Codec::VP9 => match self {
        Quality::Standard => codec::vp9::quality::STANDARD.to_string(),
        Quality::High => codec::vp9::quality::HIGH.to_string(),
        Quality::Low => codec::vp9::quality::LOW.to_string(),
      },
    }
  }
}

impl FromStr for Extension {
  type Err = ServerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      extension::MKV => Ok(Extension::Mkv),
      extension::MOV => Ok(Extension::Mov),
      extension::MP4 => Ok(Extension::Mp4),
      extension::WEBM => Ok(Extension::Webm),
      "" => Err(InputError::DataError("Extension is missing".to_string()))?,
      _ => Err(InputError::DataError(format!(
        "Extension is not supported: {s}"
      )))?,
    }
  }
}

impl Display for Extension {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Mkv => write!(f, "{}", extension::MKV),
      Self::Mov => write!(f, "{}", extension::MOV),
      Self::Mp4 => write!(f, "{}", extension::MP4),
      Self::Webm => write!(f, "{}", extension::WEBM),
    }
  }
}

impl VideoFormat {
  pub fn from_str(s: &str) -> Result<Self, ServerError> {
    let extension = Extension::from_str(s)?;

    let (muxer, mime_type) = match extension {
      Extension::Mkv => (muxer::MKV, mime_type::MKV),
      Extension::Mov => (muxer::MOV, mime_type::MOV),
      Extension::Mp4 => (muxer::MP4, mime_type::MP4),
      Extension::Webm => (muxer::WEBM, mime_type::WEBM),
    };

    Ok(Self {
      extension,
      muxer: muxer.to_string(),
      mime_type: mime_type.to_string(),
    })
  }
}
