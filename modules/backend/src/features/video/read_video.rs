use crate::core::error::ApplicationError;
use std::path::Path;

use axum::extract::Multipart;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn read_video(
  mut media_data: Multipart,
  temp_dir: &Path,
) -> Result<String, ApplicationError> {
  let mut file_path = String::new();

  while let Some(mut field) = media_data.next_field().await? {
    let field_name = field
      .name()
      .ok_or(ApplicationError::BadRequest(
        "Missing field name".to_string(),
      ))?
      .to_string();

    if field_name == "video" {
      let file_name = field
        .file_name()
        .ok_or(ApplicationError::BadRequest(
          "Missing file_name".to_string(),
        ))?
        .to_string();
      let path = temp_dir.join(file_name);

      // create local file only when needed
      let mut created_file = File::create(&path).await.map_err(|err| {
        ApplicationError::Internal(format!(
          "Failed to create temp file name: {err}"
        ))
      })?;
      file_path = path.to_string_lossy().to_string();

      // Stream chunks directly from the request network buffer into the file
      while let Some(chunk) = field.chunk().await? {
        created_file.write_all(&chunk).await.map_err(|err| {
          ApplicationError::Internal(format!(
            "Failed writing video chunk: {:?}",
            err
          ))
        })?;
      }

      // Ensure all data chunks are flushed to file
      created_file.flush().await.map_err(|err| {
        ApplicationError::BadRequest(format!(
          "Failed flushing data to file: {err}"
        ))
      })?;

      break;
    }
  }

  if file_path.is_empty() {
    return Err(ApplicationError::BadRequest(
      "Missing 'video' field".to_string(),
    ));
  }

  Ok(file_path)
}
