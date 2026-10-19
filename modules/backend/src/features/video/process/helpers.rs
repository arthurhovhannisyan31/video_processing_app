use std::collections::HashMap;
use std::sync::OnceLock;

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

type ArgsBuilder = dyn Fn(usize) -> Vec<String>;
type ArgsBuilderMap = HashMap<Operation, ArgsBuilder>;

static ARGS_BUILDER_MAP: OnceLock<ArgsBuilderMap> = OnceLock::new();
fn init_args_builder_map() -> ArgsBuilderMap {
  let mut m = HashMap::new();
  m.insert(Operation::Compress, preset::build_compress_args);
  m
}

// use std::collections::HashMap;
// use std::sync::OnceLock;
//
// // Declaring the static container
// static MY_MAP: OnceLock<HashMap<&'static str, i32>> = OnceLock::new();
//
// fn get_value(key: &str) -> Option<&i32> {
//
//     let map = MY_MAP.get_or_init(|| {
//         let mut m = HashMap::new();
//         m.insert("apple", 1);
//         m.insert("banana", 2);
//         m; // Returns the initialized map
//     });
//
//     // Read safely without locks
//     map.get(key)
// }

pub fn get_args(
  input: &str,
  output: &str,
  operation: Operation,
  codec: Codec,
  preset: Preset,
  quality: Quality,
) -> Result<Vec<String>, ServerError> {
  let mut args: Vec<String> = vec!["-i".to_string(), input.to_string()];
  // Initialize on first access
  let args_builder_map = ARGS_BUILDER_MAP.get_or_init(init_args_builder_map);
  let args_builder = args_builder_map.get(&operation);
  let preset = get_preset_by_name(operation, codec, preset, quality)?;

  args.extend(preset);
  args.extend([output.to_string()]);

  Ok(args)
}
