use std::process::Stdio;

use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

use crate::core::error::ServerError;
use crate::features::video::constants::VIDEO_API_INSPECT_TIMEOUT;

pub async fn inspect_file(file_path: &str) -> Result<Value, ServerError> {
  let mut cmd = Command::new("ffprobe");
  cmd.kill_on_drop(true);
  cmd.args([
    "-v",
    "error",
    "-show_format",
    "-show_streams",
    "-of",
    "json",
    file_path,
  ]);
  cmd.stdout(Stdio::piped());
  let ffprobe_process = cmd
    .spawn()
    .map_err(|err| ServerError::Processing(format!("Failed to spawn 'ffprobe' process: {err}",)))?;

  let output = match timeout(
    VIDEO_API_INSPECT_TIMEOUT,
    ffprobe_process.wait_with_output(),
  )
  .await
  {
    Ok(output) => output?,
    Err(_) => return Err(ServerError::Processing("ffmpeg timed out".to_string())),
  };

  if !output.status.success() {
    let err_msg = String::from_utf8_lossy(&output.stderr).into_owned();
    return Err(ServerError::Processing(format!("ffprobe error: {err_msg}")));
  }

  serde_json::from_slice(&output.stdout)
    .map_err(|_| ServerError::Processing("Invalid JSON data from ffprobe".to_string()))
}
