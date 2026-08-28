use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::log::info;
use tracing::warn;

use crate::core::error::ServerError;
use crate::features::video::constants::VIDEO_API_TIMEOUT;

pub async fn ffmpeg_runner(
  input: &str,
  output: &str,
  preset: Vec<&str>,
) -> Result<(), ServerError> {
  let mut args: Vec<&str> = vec!["-i", input];
  args.extend(preset);
  args.extend([output]);

  let mut cmd = Command::new("ffmpeg");
  cmd.kill_on_drop(true);
  cmd.args(args);
  cmd.stdout(Stdio::null());
  cmd.stderr(Stdio::piped());

  let mut ffmpeg_process = cmd
    .spawn()
    .map_err(|err| ServerError::Processing(format!("Failed to spawn 'ffmpeg' process: {err:?}")))?;

  let stderr = ffmpeg_process
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
          "progress" if value == "continue" || value == "end" => {
            info!("Progress Update -> Frame: {current_frame}, FPS: {current_fps}");
          }
          _ => {}
        }
      }
    }
    Ok(())
  });

  let status = match timeout(VIDEO_API_TIMEOUT, ffmpeg_process.wait()).await {
    Ok(res) => res?,
    Err(_) => return Err(ServerError::Processing("ffmpeg timed out".to_string())),
  };

  let _: Result<(), ServerError> = log_task.await?;

  if !status.success() {
    return Err(ServerError::Processing(format!("ffmpeg error: {}", status)));
  }

  Ok(())
}
