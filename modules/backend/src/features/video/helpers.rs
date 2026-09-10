use std::fmt::Display;
use std::io::ErrorKind;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use axum::extract::multipart::Field;
use serde::{Deserialize, Deserializer};
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

use crate::core::error::ServerError;
use crate::features::video::inspect;
use crate::features::video::process::service::MediaDataCache;
use crate::features::video::types::ReadFormDataMeta;

pub fn deserialize_string_to_type<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  D: Deserializer<'de>,
  T: FromStr,
  T::Err: Display,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<T>().map_err(serde::de::Error::custom)
}

pub fn append_path_suffix(path: &str, suffix: &str) -> Result<String, ServerError> {
  if path.is_empty() {
    return Err(ServerError::IO(io::Error::new(
      ErrorKind::NotFound,
      "File not found",
    )));
  }
  if suffix.is_empty() {
    return Err(ServerError::DataError("Suffix is empty".to_string()));
  }

  let path = Path::new(path);
  let stem = path
    .file_stem()
    .ok_or(ServerError::IO(io::Error::new(
      ErrorKind::InvalidFilename,
      "Failed to read file stem",
    )))?
    .to_str()
    .ok_or(ServerError::DataError(
      "Failed to convert file stem to string".to_string(),
    ))?;
  let extension = path
    .extension()
    .ok_or(ServerError::IO(io::Error::new(
      ErrorKind::InvalidFilename,
      "Failed to read file extension",
    )))?
    .to_str()
    .ok_or(ServerError::DataError(
      "Failed to convert file extension to string".to_string(),
    ))?;
  let parent_path = path.parent().ok_or(ServerError::IO(io::Error::new(
    ErrorKind::NotFound,
    "Failed to read file parent directory",
  )))?;
  let new_name = format!("{stem}{suffix}.{extension}");
  let output_path = parent_path.join(new_name);

  Ok(output_path.to_string_lossy().to_string())
}

pub async fn read_form_data_to_file(
  field: &mut Field<'_>,
  temp_dir: &Path,
) -> Result<ReadFormDataMeta, ServerError> {
  let mut meta = ReadFormDataMeta::default();
  let file_name_value = field
    .file_name()
    .ok_or(ServerError::DataError("Missing file_name".to_string()))?;

  meta.file_name = file_name_value.to_string();

  let safe_file_name = Path::new(file_name_value)
    .file_name()
    .ok_or(ServerError::DataError(format!(
      "Invalid filename {}",
      file_name_value
    )))?;
  let path = temp_dir.join(safe_file_name);

  // create local file only when needed
  let mut created_file = File::create(&path).await?;

  meta.local_path = path.to_string_lossy().to_string();

  let mut written_bytes: usize = 0;
  // Stream chunks directly from the request network buffer into the file
  while let Some(chunk) = field.chunk().await? {
    written_bytes += chunk.len();
    created_file.write_all(&chunk).await?;
  }

  // Ensure all data chunks are flushed to file
  created_file.flush().await?;

  if written_bytes == 0 {
    return Err(ServerError::DataError("Form data is empty".to_string()));
  }

  Ok(meta)
}

pub async fn get_file_duration(
  file_name: &str,
  local_path: &str,
  media_data_cache: &MediaDataCache,
  video_inspect_timeout: Duration,
) -> Result<f64, ServerError> {
  let duration = match media_data_cache.get(&file_name.to_string()) {
    Some(meta) => meta.duration_seconds,
    None => {
      let inspection_raw_data =
        inspect::ffprobe_runner::inspect_file(local_path, video_inspect_timeout).await?;
      let media_meta_data = inspect::ffprobe_mapper::map_media_meta(inspection_raw_data)?;
      media_data_cache.insert(file_name.to_string(), media_meta_data.clone());
      media_meta_data.duration_seconds
    }
  };

  if duration <= 0.0 {
    Err(ServerError::Processing(
      "File has zero duration".to_string(),
    ))?;
  }

  Ok(duration)
}

#[cfg(test)]
mod tests {
  use axum::body::Body;
  use axum::extract::multipart::Multipart;
  use axum::extract::{FromRequest, Request};
  use tempfile::tempdir;

  use super::*;

  #[test]
  fn test_append_path_suffix() {
    assert!(append_path_suffix("", "").is_err());
    assert!(append_path_suffix("/", "").is_err());
    assert!(append_path_suffix("", "/").is_err());
    assert!(append_path_suffix("/", "temp").is_err());
    assert!(append_path_suffix("/dir", "temp").is_err());
    assert_eq!(
      append_path_suffix("/file.ext", "-temp").unwrap(),
      "/file-temp.ext".to_string()
    );
    assert_eq!(
      append_path_suffix("../file.ext", "-temp").unwrap(),
      "../file-temp.ext".to_string()
    );
  }

  async fn multipart_from_field(filename: Option<&str>, content: &[u8]) -> Multipart {
    let boundary = "test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    match filename {
      Some(name) => body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"video\"; filename=\"{name}\"\r\n")
          .as_bytes(),
      ),
      None => body.extend_from_slice(b"Content-Disposition: form-data; name=\"video\"\r\n"),
    }
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let request = Request::builder()
      .header(
        "content-type",
        format!("multipart/form-data; boundary={boundary}"),
      )
      .body(Body::from(body))
      .unwrap();

    Multipart::from_request(request, &()).await.unwrap()
  }

  #[tokio::test]
  async fn test_read_video_to_file_success() {
    let temp_dir = tempdir().unwrap();
    let content = b"fake video bytes";
    let mut multipart = multipart_from_field(Some("video.mp4"), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let meta = read_form_data_to_file(&mut field, temp_dir.path())
      .await
      .unwrap();

    assert!(Path::new(&meta.local_path).starts_with(temp_dir.path()));
    let saved = tokio::fs::read(&meta.local_path).await.unwrap();
    assert_eq!(saved, content);
  }

  #[tokio::test]
  async fn test_read_video_to_file_missing_filename() {
    let temp_dir = tempdir().unwrap();
    let mut multipart = multipart_from_field(None, b"data").await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let result = read_form_data_to_file(&mut field, temp_dir.path()).await;

    assert!(matches!(result, Err(ServerError::DataError(msg)) if msg == "Missing file_name"));
  }

  #[tokio::test]
  async fn test_read_video_to_file_strips_path_traversal() {
    let temp_dir = tempdir().unwrap();
    let content = b"data";
    let mut multipart = multipart_from_field(Some("../../etc/passwd"), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let meta = read_form_data_to_file(&mut field, temp_dir.path())
      .await
      .unwrap();

    let saved_path = Path::new(&meta.local_path);
    assert_eq!(saved_path.parent().unwrap(), temp_dir.path());
    assert_eq!(saved_path.file_name().unwrap(), "passwd");
  }

  #[tokio::test]
  async fn test_read_video_to_file_invalid_filename() {
    let temp_dir = tempdir().unwrap();
    let mut multipart = multipart_from_field(Some(".."), b"data").await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let result = read_form_data_to_file(&mut field, temp_dir.path()).await;

    assert!(matches!(result, Err(ServerError::DataError(_))));
  }
}
