use crate::core::error::ApplicationError;

use axum::extract::multipart::Field;
use serde_json::Value;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn ffprobe_runner<'a>(
  mut video_field: Field<'a>,
) -> Result<Value, ApplicationError> {
  let mut ffprobe_process = tokio::task::spawn_blocking(move || {
    Command::new("ffprobe")
      .args([
        "-v",
        "error",
        "-show_format",
        "-show_streams",
        "-of",
        "json",
        "-",
      ])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
  })
  .await
  .map_err(|err| {
    ApplicationError::Internal(format!("Failed to spawn ffprobe: {err}"))
  })?
  .map_err(|err| {
    ApplicationError::Internal(format!(
      "Failed executing ffprobe binary: {err}"
    ))
  })?;

  let mut stdin =
    ffprobe_process
      .stdin
      .take()
      .ok_or(ApplicationError::Internal(
        "Failed to open stdin".to_string(),
      ))?;

  while let Some(chunk) = video_field.chunk().await? {
    if stdin.write_all(&chunk).await.is_err() {
      break;
    }
  }

  stdin.flush().await.map_err(|err| {
    ApplicationError::BadRequest(format!(
      "Failed flushing data to stdin: {err}"
    ))
  })?;

  drop(stdin);

  let output = match ffprobe_process.wait_with_output().await {
    Ok(out) if out.status.success() => out,
    Ok(out) => {
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
