use crate::core::error::ApplicationError;
use tokio::process::Command;

pub async fn ffmpeg_runner(
  input: &str,
  output: &str,
  preset: Vec<&str>,
) -> Result<(), ApplicationError> {
  let mut args: Vec<&str> = vec!["-i", input];
  args.extend(preset);
  args.extend([output]);

  let output = Command::new("ffmpeg")
    .kill_on_drop(true)
    .args(args)
    .output()
    .await
    .map_err(|err| {
      ApplicationError::Internal(format!(
        "Failed executing 'ffmpeg' binary: {err}"
      ))
    })?;

  if !output.status.success() {
    let err_msg = String::from_utf8_lossy(&output.stderr).into_owned();
    return Err(ApplicationError::BadRequest(format!(
      "ffmpeg error: {err_msg}"
    )));
  }

  Ok(())
}
