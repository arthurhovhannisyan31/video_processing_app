use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{error, warn};
use uuid::Uuid;

use crate::core::error::ServerError;
use crate::features::video::constants::VIDEO_API_PROCESS_TIMEOUT;
use crate::features::video::state::{VideoState, VideoStateMessage, VideoStateProgress};

pub async fn process_file(
  input: &str,
  output: &str,
  preset: Vec<&str>,
  video_state: Arc<VideoState>,
  user_id: Uuid,
  duration_seconds: f64,
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

  let log_task: JoinHandle<Result<(), ServerError>> = tokio::spawn(async move {
    while let Some(line) = error_lines.next_line().await? {
      let line = line.trim();

      if let Some((key, value)) = line.split_once('=') {
        match key {
          "out_time_ms" => {
            if value == "N/A" {
              continue;
            }

            let out_time_microseconds: i64 = value.parse().map_err(ServerError::ParseIntError)?;
            let out_time_seconds: f64 = out_time_microseconds as f64 / 1000000.0;
            let progress_value = out_time_seconds / duration_seconds;
            let message = VideoStateMessage {
              id: user_id,
              message: VideoStateProgress {
                value: progress_value,
                done: false,
              },
            };
            if let Err(err) = video_state.channel_tx.send(message) {
              warn!("Error while sending message to video state stream: {err}");
            }
          }
          "progress" if value == "end" => {
            let message = VideoStateMessage {
              id: user_id,
              message: VideoStateProgress {
                value: 1.0,
                done: true,
              },
            };
            if let Err(err) = video_state.channel_tx.send(message) {
              warn!("Error while sending message to video state stream: {err}");
            }
          }
          _ => {}
        }
      }
    }
    Ok(())
  });

  let status = match timeout(VIDEO_API_PROCESS_TIMEOUT, ffmpeg_process.wait()).await {
    Ok(res) => res?,
    Err(_) => return Err(ServerError::Processing("ffmpeg timed out".to_string())),
  };

  if let Err(err) = log_task.await? {
    // Do not return if logging failed
    error!("Failed to log processing progress {err:?}");
  }

  if !status.success() {
    return Err(ServerError::Processing(format!("ffmpeg error: {}", status)));
  }

  Ok(())
}
