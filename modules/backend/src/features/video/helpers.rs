use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;

use axum::extract::multipart::Field;
use serde::{Deserialize, Deserializer};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::core::error::{ApplicationError, ServerError};

pub fn deserialize_string_to_type<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  D: Deserializer<'de>,
  T: FromStr,
  T::Err: Display,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<T>().map_err(serde::de::Error::custom)
}

pub fn append_path_suffix(path: &str, suffix: &str) -> Result<String, ApplicationError> {
  if path.is_empty() {
    return Err(ApplicationError::Internal("Path is empty".to_string()));
  }
  if suffix.is_empty() {
    return Err(ApplicationError::Internal("Suffix is empty".to_string()));
  }

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

pub async fn read_video_to_file(
  field: &mut Field<'_>,
  temp_dir: &Path,
) -> Result<String, ServerError> {
  let file_name_value = field
    .file_name()
    .ok_or(ServerError::DataError("Missing file_name".to_string()))?;
  let safe_file_name = Path::new(file_name_value)
    .file_name()
    .ok_or(ServerError::DataError(format!(
      "Invalid filename {}",
      file_name_value
    )))?;
  let path = temp_dir.join(safe_file_name);

  // create local file only when needed
  let mut created_file = File::create(&path).await?;
  let file_path = path.to_string_lossy().to_string();

  let mut written_bytes: usize = 0;
  // Stream chunks directly from the request network buffer into the file
  while let Some(chunk) = field.chunk().await? {
    written_bytes += chunk.len();
    created_file.write_all(&chunk).await?;
  }

  // Ensure all data chunks are flushed to file
  created_file.flush().await?;

  if written_bytes == 0 {
    return Err(ServerError::DataError(
      "Uploaded video file is empty".to_string(),
    ));
  }

  if file_path.is_empty() {
    return Err(ServerError::DataError("Missing 'video' field".to_string()));
  }

  Ok(file_path)
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

    let file_path = read_video_to_file(&mut field, temp_dir.path())
      .await
      .unwrap();

    assert!(Path::new(&file_path).starts_with(temp_dir.path()));
    let saved = tokio::fs::read(&file_path).await.unwrap();
    assert_eq!(saved, content);
  }

  #[tokio::test]
  async fn test_read_video_to_file_missing_filename() {
    let temp_dir = tempdir().unwrap();
    let mut multipart = multipart_from_field(None, b"data").await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let result = read_video_to_file(&mut field, temp_dir.path()).await;

    assert!(matches!(result, Err(ServerError::DataError(msg)) if msg == "Missing file_name"));
  }

  #[tokio::test]
  async fn test_read_video_to_file_strips_path_traversal() {
    let temp_dir = tempdir().unwrap();
    let content = b"data";
    let mut multipart = multipart_from_field(Some("../../etc/passwd"), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let file_path = read_video_to_file(&mut field, temp_dir.path())
      .await
      .unwrap();

    let saved_path = Path::new(&file_path);
    assert_eq!(saved_path.parent().unwrap(), temp_dir.path());
    assert_eq!(saved_path.file_name().unwrap(), "passwd");
  }

  #[tokio::test]
  async fn test_read_video_to_file_invalid_filename() {
    let temp_dir = tempdir().unwrap();
    let mut multipart = multipart_from_field(Some(".."), b"data").await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let result = read_video_to_file(&mut field, temp_dir.path()).await;

    assert!(matches!(result, Err(ServerError::DataError(_))));
  }
}
