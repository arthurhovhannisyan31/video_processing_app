use crate::core::error::ApplicationError;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub async fn ffmpeg_runner(
  input: &str,
  output: &str,
  preset: Vec<&str>,
) -> Result<(), ApplicationError> {
  let mut args: Vec<&str> = vec!["-i", input];
  args.extend(preset);
  args.extend([output]);

  let mut child_process = Command::new("ffmpeg")
    .kill_on_drop(true)
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|err| {
      ApplicationError::Internal(format!(
        "Failed to spawn 'ffmpeg' process: {err:?}"
      ))
    })?;

  let stdout =
    child_process
      .stdout
      .take()
      .ok_or(ApplicationError::Internal(
        "Missing ffmpeg stdout".to_string(),
      ))?;
  let stderr =
    child_process
      .stderr
      .take()
      .ok_or(ApplicationError::Internal(
        "Missing ffmpeg stderr".to_string(),
      ))?;

  let mut progress_lines = BufReader::new(stdout).lines();
  let mut error_lines = BufReader::new(stderr).lines();

  let progress_task = tokio::spawn(async move {
    while let Some(line) = progress_lines.next_line().await? {
      // white to db
      println!("Progress: {line:?}");
    }
    Ok(())
  });

  let stderr_task = tokio::spawn(async move {
    while let Some(line) = error_lines.next_line().await? {
      // write to db
      println!("Progress: {line:?}");
    }
    Ok(())
  });

  let status = child_process.wait().await?;
  let _: Result<(), ApplicationError> = progress_task.await?;
  let _: Result<(), ApplicationError> = stderr_task.await?;

  if !status.success() {
    return Err(ApplicationError::BadRequest(format!(
      "ffmpeg error: {}",
      status.to_string()
    )));
  }

  Ok(())
}
