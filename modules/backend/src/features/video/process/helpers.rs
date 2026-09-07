use std::str::FromStr;

use crate::core::error::ServerError;
use crate::features::video::process::configs::{OperationType, preset};

pub fn get_preset_by_name<'a>(operation: &str) -> Result<Vec<&'a str>, ServerError> {
  let operation_type = OperationType::from_str(operation)?;

  match operation_type {
    OperationType::Compress => Ok(preset::compress()),
  }
}

pub fn get_args<'a>(
  input: &'a str,
  output: &'a str,
  operation: &'a str,
) -> Result<Vec<&'a str>, ServerError> {
  let mut args: Vec<&str> = vec!["-i", input];
  let preset = get_preset_by_name(operation)?;

  args.extend(preset);
  args.extend([output]);

  Ok(args)
}
