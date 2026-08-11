use serde_json::Value;
use tokio::process::Command;

use crate::core::error::ApplicationError;

pub async fn ffprobe_runner(file_path: &str) -> Result<Value, ApplicationError> {
  let output = Command::new("ffprobe")
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
    .output()
    .await
    .map_err(|err| {
      ApplicationError::BadRequest(format!("Failed executing ffprobe binary: {err}",))
    })?;

  if !output.status.success() {
    let err_msg = String::from_utf8_lossy(&output.stderr).into_owned();
    return Err(ApplicationError::BadRequest(format!(
      "ffprobe error: {err_msg}"
    )));
  }

  serde_json::from_slice(&output.stdout)
    .map_err(|_| ApplicationError::Internal("Invalid JSON data from ffprobe".to_string()))
}
