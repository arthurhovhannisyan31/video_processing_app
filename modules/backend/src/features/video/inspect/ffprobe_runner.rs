use std::process::Stdio;

use serde_json::Value;
use sqlx::__rt::timeout;
use tokio::process::Command;

use crate::core::error::ServerError;
use crate::features::video::constants::VIDEO_API_TIMEOUT;

pub async fn ffprobe_runner(file_path: &str) -> Result<Value, ServerError> {
  let ffprobe_process = Command::new("ffprobe")
    .kill_on_drop(true)
    .args([
      "-v",
      "error",
      "-show_format",
      "-show_streams",
      "-of",
      "json",
      file_path,
    ])
    .stdout(Stdio::piped())
    .spawn()
    .map_err(|err| ServerError::Processing(format!("Failed to spawn 'ffprobe' process: {err}",)))?;

  let output = match timeout(VIDEO_API_TIMEOUT, ffprobe_process.wait_with_output()).await {
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
