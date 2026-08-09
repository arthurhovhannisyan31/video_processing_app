use crate::core::error::{ApplicationError, ServerError};
use anyhow::anyhow;
use axum::extract::multipart::Field;
use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub fn deserialize_string_to_type<'de, D, T>(
  deserializer: D,
) -> Result<T, D::Error>
where
  D: Deserializer<'de>,
  T: FromStr,
  T::Err: Display,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<T>().map_err(serde::de::Error::custom)
}

// TODO Tests
pub fn append_path_suffix(
  path: &str,
  suffix: &str,
) -> Result<String, ApplicationError> {
  let path = Path::new(path);
  let stem = path
    .file_stem()
    .ok_or(ApplicationError::Internal(
      "Failed to read file stem".to_string(),
    ))?
    .to_str()
    .ok_or(ApplicationError::Internal(
      "Failed to convert file stem to string".to_string(),
    ))?;
  let extension = path
    .extension()
    .ok_or(ApplicationError::Internal(
      "Failed to read file extension".to_string(),
    ))?
    .to_str()
    .ok_or(ApplicationError::Internal(
      "Failed to convert file extension to string".to_string(),
    ))?;
  let parent_path = path.parent().ok_or(ApplicationError::Internal(
    "Failed to read file parent directory".to_string(),
  ))?;
  let new_name = format!("{stem}{suffix}.{extension}");
  let output_path = parent_path.join(new_name);

  Ok(output_path.to_string_lossy().to_string())
}

// TODO Tests
pub async fn read_video_to_file(
  field: &mut Field<'_>,
  temp_dir: &Path,
) -> Result<String, ServerError> {
  let file_name = field
    .file_name()
    .ok_or(ServerError::DataError("Missing file_name".to_string()))?;
  let path = temp_dir.join(file_name);

  // create local file only when needed
  let mut created_file = File::create(&path).await?;
  let file_path = path.to_string_lossy().to_string();

  // Stream chunks directly from the request network buffer into the file
  while let Some(chunk) = field.chunk().await? {
    created_file.write_all(&chunk).await?;
  }

  // Ensure all data chunks are flushed to file
  created_file.flush().await?;

  if file_path.is_empty() {
    return Err(ServerError::OtherError(anyhow!(
      "Missing 'video' field".to_string()
    )));
  }

  Ok(file_path)
}
