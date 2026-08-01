use crate::core::error::ApplicationError;

use serde_json::Value;
use std::process::Command;

pub async fn ffprobe_runner(
  file_path: String,
) -> Result<Value, ApplicationError> {
  let probe_result = tokio::task::spawn_blocking(move || {
    Command::new("ffprobe")
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
  })
  .await;

  let output = match probe_result {
    Ok(Ok(out)) if out.status.success() => out,
    Ok(Ok(out)) => {
      let err_msg = String::from_utf8_lossy(&out.stderr).into_owned();
      return Err(ApplicationError::BadRequest(format!(
        "ffprobe error: {err_msg}"
      )));
    }
    _ => {
      return Err(ApplicationError::BadRequest(
        "Failed executing ffprobe binary".to_string(),
      ));
    }
  };

  serde_json::from_slice(&output.stdout).map_err(|_| {
    ApplicationError::Internal("Invalid JSON data from ffprobe".to_string())
  })
}
