use crate::core::error::ServerError;
use crate::features::video::process::configs::preset;
use crate::features::video::process::types::{Codec, Operation, Preset, Quality};

pub fn get_preset_by_name(
  operation: Operation,
  codec: Codec,
  preset: Preset,
  quality: Quality,
) -> Result<Vec<String>, ServerError> {
  match operation {
    Operation::Compress => Ok(preset::build_compress_args(codec, preset, quality)),
  }
}

pub fn get_args(
  input: &str,
  output: &str,
  operation: Operation,
  codec: Codec,
  preset: Preset,
  quality: Quality,
) -> Result<Vec<String>, ServerError> {
  let mut args: Vec<String> = vec!["-i".to_string(), input.to_string()];
  let preset = get_preset_by_name(operation, codec, preset, quality)?;

  args.extend(preset);
  args.extend([output.to_string()]);

  Ok(args)
}
