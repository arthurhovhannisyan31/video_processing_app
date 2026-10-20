use crate::core::error::ServerError;
use crate::features::video::process::preset::{ARGS_BUILDER_MAP, init_args_builder};
use crate::features::video::process::types::ProcessVideoMeta;

pub fn get_args(meta: &ProcessVideoMeta, output: &str) -> Result<Vec<String>, ServerError> {
  let mut args: Vec<String> = vec!["-i".to_string(), meta.local_path.to_string()];

  let map = ARGS_BUILDER_MAP.get_or_init(init_args_builder);
  let Some(builder) = map.get(&meta.codec) else {
    return Err(ServerError::Processing(format!(
      "Cannot find arguments builder for codec: {}",
      meta.codec
    )));
  };

  let preset = builder(meta)?;
  args.extend(preset);
  args.extend([output.to_string()]);

  Ok(args)
}
