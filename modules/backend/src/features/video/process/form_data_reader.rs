use std::path::Path;

use axum::extract::Multipart;

use crate::core::error::ServerError;
use crate::features::video::helpers::read_video_to_file;
use crate::features::video::process::types::ProcessVideoMeta;

pub async fn read(
  mut media_data: Multipart,
  temp_dir: &Path,
) -> Result<ProcessVideoMeta, ServerError> {
  let mut meta = ProcessVideoMeta::default();

  while let Some(mut field) = media_data.next_field().await? {
    let field_name = field
      .name()
      .ok_or(ServerError::DataError("Missing field name".to_string()))?
      .to_string();

    match field_name.as_str() {
      "video" => {
        meta.file_path = read_video_to_file(&mut field, temp_dir).await?;
      }
      "operation" => {
        meta.command = field.text().await?;
      }
      _ => {}
    }
  }

  if meta.file_path.is_empty() {
    return Err(ServerError::DataError("Missing 'video' field".to_string()));
  }

  Ok(meta)
}
