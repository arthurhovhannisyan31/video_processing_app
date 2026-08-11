use std::path::Path;
use std::str::FromStr;

use anyhow::anyhow;
use axum::extract::Multipart;

use crate::core::error::ServerError;
use crate::features::video::helpers::read_video_to_file;
use crate::features::video::process::configs::FieldName;

pub async fn read(mut media_data: Multipart, temp_dir: &Path) -> Result<String, ServerError> {
  let mut file_path = String::new();

  while let Some(mut field) = media_data.next_field().await? {
    let field_name = field
      .name()
      .ok_or(ServerError::OtherError(anyhow!(
        "Missing field name".to_string(),
      )))?
      .to_string();

    if FieldName::from_str(&field_name)? == FieldName::Video {
      file_path = read_video_to_file(&mut field, temp_dir).await?;
      break;
    }
  }

  if file_path.is_empty() {
    return Err(ServerError::DataError("Missing 'video' field".to_string()));
  }

  Ok(file_path)
}
