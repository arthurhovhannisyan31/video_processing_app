use std::process::Stdio;

use anyhow::anyhow;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::core::error::ServerError;

pub async fn ffmpeg_runner(
  input: &str,
  output: &str,
  preset: Vec<&str>,
) -> Result<(), ServerError> {
  let mut args: Vec<&str> = vec!["-i", input];
  args.extend(preset);
  args.extend([output]);

  let mut child_process = Command::new("ffmpeg")
    .kill_on_drop(true)
    .args(args)
    .stdout(Stdio::null()) // FFmpeg output file mode doesn't use stdout
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|err| ServerError::OtherError(anyhow!("Failed to spawn 'ffmpeg' process: {err:?}")))?;

  let stderr = child_process
    .stderr
    .take()
    .ok_or(ServerError::Processing("Missing ffmpeg stderr".to_string()))?;

  let mut error_lines = BufReader::new(stderr).lines();

  let log_task = tokio::spawn(async move {
    let mut current_frame = "0".to_string();
    let mut current_fps = "0".to_string();

    while let Some(line) = error_lines.next_line().await? {
      let line = line.trim();

      if let Some((key, value)) = line.split_once('=') {
        match key {
          "frame" => current_frame = value.to_string(),
          "fps" => current_fps = value.to_string(),
          "progress" => {
            println!("DB update -> Frame: {current_frame}, FPS: {current_fps}, Status: {value}");
          }
          _ => {}
        }
      } else if !line.is_empty() {
        println!("FFmpeg Log/Error: {}", line);
      }
    }
    Ok(())
  });

  let status = child_process.wait().await?;
  let _: Result<(), ServerError> = log_task.await?;

  if !status.success() {
    return Err(ServerError::Processing(format!(
      "ffmpeg error: {}",
      status.to_string()
    )));
  }

  Ok(())
}
