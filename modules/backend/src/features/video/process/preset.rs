use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::error::ServerError;
use crate::features::video::process::configs::codec;
use crate::features::video::process::types::{Codec, Operation, ProcessVideoMeta};

type ArgsBuilderFn = fn(&ProcessVideoMeta) -> Result<Vec<String>, ServerError>;
type ArgsBuilderMap = HashMap<Codec, ArgsBuilderFn>;
pub static ARGS_BUILDER_MAP: OnceLock<ArgsBuilderMap> = OnceLock::new();

pub fn get_x264_preset(meta: &ProcessVideoMeta) -> Result<Vec<String>, ServerError> {
  let ProcessVideoMeta {
    operation,
    codec,
    preset,
    quality,
    format,
    ..
  } = meta;

  match operation {
    Operation::Compress => Ok(codec::x264::build_args(
      preset,
      quality.to_string(codec),
      format,
    )),
  }
}

pub fn get_vp9_preset(meta: &ProcessVideoMeta) -> Result<Vec<String>, ServerError> {
  let ProcessVideoMeta {
    operation,
    codec,
    quality,
    format,
    ..
  } = meta;

  match operation {
    Operation::Compress => Ok(codec::vp9::build_args(quality.to_string(codec), format)),
  }
}

pub fn init_args_builder() -> ArgsBuilderMap {
  let mut m: ArgsBuilderMap = HashMap::new();
  m.insert(Codec::X264, get_x264_preset as ArgsBuilderFn);
  m.insert(Codec::VP9, get_vp9_preset as ArgsBuilderFn);
  m
}
