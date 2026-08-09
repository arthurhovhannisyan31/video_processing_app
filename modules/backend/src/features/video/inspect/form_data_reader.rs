use std::path::Path;

use anyhow::anyhow;
use axum::extract::Multipart;

use crate::core::error::ServerError;
use crate::features::video::helpers::read_video_to_file;

pub async fn read(mut media_data: Multipart, temp_dir: &Path) -> Result<String, ServerError> {
  let mut file_path = String::new();

  while let Some(mut field) = media_data.next_field().await? {
    let field_name = field
      .name()
      .ok_or(ServerError::OtherError(anyhow!(
        "Missing field name".to_string(),
      )))?
      .to_string();

    if field_name == "video" {
      file_path = read_video_to_file(&mut field, temp_dir).await?;
      break;
    }
  }

  Ok(file_path)
}
