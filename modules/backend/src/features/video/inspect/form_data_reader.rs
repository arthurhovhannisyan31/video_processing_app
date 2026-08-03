use crate::core::error::ApplicationError;
use crate::features::video::helpers::read_video_to_file;
use axum::extract::Multipart;
use std::path::Path;

pub async fn read(
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
      file_path = read_video_to_file(&mut field, temp_dir).await?;
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
