use crate::core::error::ApplicationError;
use serde_json::Value;
use tokio::process::Command;

pub async fn ffprobe_runner(
  file_path: &str,
) -> Result<Value, ApplicationError> {
  let probe_output = Command::new("ffprobe")
    .args([
      "-v",
      "error",
      "-show_format",
      "-show_streams",
      "-of",
      "json",
      &file_path,
    ])
    .output()
    .await
    .map_err(|err| {
      ApplicationError::BadRequest(format!(
        "Failed executing ffprobe binary: {err}",
      ))
    })?;

  if !probe_output.status.success() {
    let err_msg = String::from_utf8_lossy(&probe_output.stderr).into_owned();
    return Err(ApplicationError::BadRequest(format!(
      "ffprobe error: {err_msg}"
    )));
  }

  serde_json::from_slice(&probe_output.stdout).map_err(|_| {
    ApplicationError::Internal("Invalid JSON data from ffprobe".to_string())
  })
}
