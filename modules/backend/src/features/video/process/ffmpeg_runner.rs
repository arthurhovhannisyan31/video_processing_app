use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::warn;
use uuid::Uuid;

use crate::core::error::ServerError;
use crate::features::video::constants::{VIDEO_MAX_PROGRESS_VALUE, VIDEO_MIN_PROGRESS_VALUE};
use crate::features::video::model::VideoStateProgress;
use crate::features::video::process::service::VideoWsConnectionsMap;

pub async fn process_file(
  args: Vec<&str>,
  file_name: &str,
  connections_map: &VideoWsConnectionsMap,
  user_id: Uuid,
  duration_seconds: f64,
  process_timeout: Duration,
) -> Result<(), ServerError> {
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
  let file_name_str = file_name.to_string();

  let process_result = timeout(process_timeout, async {
    loop {
      tokio::select! {
        // Drain the stderr pipe lines to prevent buffer overflow and drop
        line_res = error_lines.next_line() => {
          match line_res {
            Ok(Some(line)) => {
              let line = line.trim();
              if let Some((key, value)) = line.split_once('=') {
                let mut message: Option<VideoStateProgress> = None;

                match key {
                  "out_time_ms" => {
                    if value == "N/A" { continue; }
                    let out_time_microseconds: i64 = value.parse().map_err(ServerError::ParseIntError)?;
                    let out_time_seconds: f64 = out_time_microseconds as f64 / 1_000_000.0;
                    let progress_value = (out_time_seconds / duration_seconds)
                      .clamp(VIDEO_MIN_PROGRESS_VALUE, VIDEO_MAX_PROGRESS_VALUE);

                    message = Some(VideoStateProgress {
                      file_name: file_name_str.clone(),
                      value: progress_value,
                      done: false,
                    });
                  }
                  "progress" if value == "end" => {
                    message = Some(VideoStateProgress {
                      file_name: file_name_str.clone(),
                      value: 1.0,
                      done: true,
                    });
                  }
                  _ => {}
                }

                if let Some(message) = message {
                  let tx = connections_map.read().get(&user_id).cloned();
                  if let Some(tx) = tx && let Err(err) = tx.send(message).await {
                    warn!("Error while sending message to video state stream: {err}");
                  }
                }
              }
            }
            Ok(None) => {
              // Pipe closed
              continue;
            }
            Err(err) => {
              warn!("Failed to read processing progress line: {err:?}");
            }
          }
        }
        // Wait for the exit status
        status_res = ffmpeg_process.wait() => {
          return status_res.map_err(|e| ServerError::Processing(e.to_string()));
        }
      }
    }
  }).await;

  // Handle timeout or underlying processing result
  let status = match process_result {
    Ok(res) => res?,
    Err(_) => return Err(ServerError::Processing("ffmpeg timed out".to_string())),
  };

  if !status.success() {
    return Err(ServerError::Processing(format!("ffmpeg error: {}", status)));
  }

  Ok(())
}
