use std::process::Stdio;

use tokio::process::Command;
use tokio::time::timeout;

use crate::core::error::ServerError;
use crate::features::video::constants::VIDEO_API_INSPECT_TIMEOUT;

pub async fn inspect_file_size(file_path: &str) -> Result<f32, ServerError> {
  let mut cmd = Command::new("ffprobe");
  cmd.kill_on_drop(true);
  cmd.args([
    "-v",
    "error",
    "-show_entries",
    "format=duration",
    "-of",
    "default=noprint_wrappers=1:nokey=1",
    file_path,
  ]);
  cmd.stdout(Stdio::piped());
  let ffprobe_process = cmd
    .spawn()
    .map_err(|err| ServerError::Processing(format!("Failed to spawn 'ffprobe' process: {err}",)))?;

  let output = match timeout(
    VIDEO_API_INSPECT_TIMEOUT,
    ffprobe_process.wait_with_output(),
  )
  .await
  {
    Ok(output) => output?,
    Err(_) => return Err(ServerError::Processing("ffmpeg timed out".to_string())),
  };

  if !output.status.success() {
    let err_msg = String::from_utf8_lossy(&output.stderr).into_owned();
    return Err(ServerError::Processing(format!("ffprobe error: {err_msg}")));
  }

  let stdout_str = std::str::from_utf8(&output.stdout).map_err(|err| {
    ServerError::Processing(format!("ffprobe output contained invalid UTF-8: {err}"))
  })?;

  let duration_seconds: f32 = stdout_str
    .trim()
    .parse()
    .map_err(ServerError::ParseFloatError)?;

  Ok(duration_seconds)
}
