use std::path::Path;
use std::str::FromStr;

use axum::extract::Multipart;

use crate::core::error::ServerError;
use crate::features::video::helpers::read_form_data_to_file;
use crate::features::video::process::configs::FieldName;
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

    let field_name = FieldName::from_str(&field_name)?;

    match field_name {
      FieldName::Video => {
        let read_form_data_meta = read_form_data_to_file(&mut field, temp_dir).await?;
        meta.file_name = read_form_data_meta.file_name;
        meta.local_path = read_form_data_meta.local_path;
      }
      FieldName::Operation => {
        meta.operation = field.text().await?;
      }
    }
  }

  if meta.file_name.is_empty() {
    return Err(ServerError::DataError("Missing 'video' field".to_string()));
  }

  Ok(meta)
}
