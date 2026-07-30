use crate::core::error::ApplicationError;

use axum::extract::Multipart;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn read_to_file(
  mut multipart: Multipart,
) -> Result<String, ApplicationError> {
  let temp_file = tempfile::NamedTempFile::new().map_err(|err| {
    ApplicationError::Internal(format!("Failed to create temp file: {err}"))
  })?;
  let temp_path = temp_file.into_temp_path();
  let mut target_file = File::create(&temp_path).await.map_err(|err| {
    ApplicationError::Internal(format!("Failed to open temp file: {err}"))
  })?;
  let mut file_found = false;

  while let Some(mut field) = multipart.next_field().await? {
    let field_name = field
      .name()
      .ok_or(ApplicationError::BadRequest(
        "Missing field name".to_string(),
      ))?
      .to_string();

    if field_name == "video" {
      file_found = true;

      while let Some(chunk) = field.chunk().await? {
        // Stream chunks directly from the request network buffer into the file
        target_file.write_all(&chunk).await.map_err(|err| {
          ApplicationError::Internal(format!(
            "Failed writing video chunk: {:?}",
            err
          ))
        })?;
      }

      break;
    }
  }

  if !file_found {
    return Err(ApplicationError::BadRequest(
      "Missing 'video' field".to_string(),
    ));
  }

  // Ensure all data is flushed to file
  target_file.flush().await.map_err(|err| {
    ApplicationError::BadRequest(format!("Failed flushing data to disk: {err}"))
  })?;

  Ok(temp_path.to_string_lossy().to_string())
}
