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
use uuid::Uuid;

use crate::core::error::{InputError, ServerError};
use crate::core::hash::calculate_hash;
use crate::features::video::cache::MediaDataCache;
use crate::features::video::constants::FILE_EXTENSIONS_WHITE_LIST;
use crate::features::video::inspect;
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

pub fn deserialize_string_to_option_type<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
  D: Deserializer<'de>,
  T: FromStr,
  T::Err: Display,
{
  let s = String::deserialize(deserializer)?;
  let value = s.parse::<T>().map_err(serde::de::Error::custom)?;
  Ok(Some(value))
}

pub fn append_path_suffix(path: &str, suffix: &str) -> Result<String, ServerError> {
  if path.is_empty() {
    return Err(ServerError::IO(io::Error::new(
      ErrorKind::NotFound,
      "File not found",
    )));
  }
  if suffix.is_empty() {
    Err(InputError::DataError("Suffix is empty".to_string()))?;
  }

  let path = Path::new(path);
  let stem = path
    .file_stem()
    .ok_or(InputError::DataError(
      "Failed to read file stem".to_string(),
    ))?
    .to_str()
    .ok_or(InputError::DataError(
      "Failed to convert file stem to string".to_string(),
    ))?;
  let extension = path
    .extension()
    .ok_or(InputError::DataError(
      "Failed to read file extension".to_string(),
    ))?
    .to_str()
    .ok_or(InputError::DataError(
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

pub async fn read_form_data_meta(field: &mut Field<'_>) -> Result<ReadFormDataMeta, ServerError> {
  let mut meta = ReadFormDataMeta::default();
  let original_file_name = field
    .file_name()
    .ok_or(InputError::DataError("Missing file name".to_string()))?;

  meta.original_file_name = original_file_name.to_string();

  let file_path = Path::new(original_file_name);
  let file_extension = file_path
    .extension()
    .ok_or(InputError::DataError(format!(
      "Missing file extension {}",
      original_file_name
    )))?
    .to_string_lossy()
    .to_string();

  // whitelist wrong file extensions
  if !FILE_EXTENSIONS_WHITE_LIST.contains(&file_extension.as_str()) {
    Err(InputError::DataError(format!(
      "File extension is not supported: {}",
      file_extension
    )))?;
  }

  let safe_file_name = Path::new(original_file_name)
    .file_name()
    .ok_or(InputError::DataError(format!(
      "Invalid filename {}",
      original_file_name
    )))?;

  meta.safe_file_name = safe_file_name.to_string_lossy().to_string();

  Ok(meta)
}

pub async fn read_form_data_to_file(
  field: &mut Field<'_>,
  form_data_meta: &ReadFormDataMeta,
  temp_dir: &Path,
) -> Result<String, ServerError> {
  let path = temp_dir.join(form_data_meta.safe_file_name.clone());

  // create local file only when needed
  let mut created_file = File::create(&path).await?;

  let mut written_bytes: usize = 0;
  // Stream chunks directly from the request network buffer into the file
  while let Some(chunk) = field.chunk().await.map_err(InputError::Multipart)? {
    written_bytes += chunk.len();
    created_file.write_all(&chunk).await?;
  }

  // Ensure all data chunks are flushed to file
  created_file.flush().await?;

  if written_bytes == 0 {
    Err(InputError::DataError("Form data is empty".to_string()))?;
  }

  let local_file_path = path.to_string_lossy().to_string();

  Ok(local_file_path)
}

pub async fn get_file_duration(
  file_name: &str,
  user_id: Uuid,
  local_path: &str,
  media_data_cache: &MediaDataCache,
  video_inspect_timeout: Duration,
) -> Result<f64, ServerError> {
  let file_hash = calculate_hash(&format!("{}{}", file_name, user_id));

  let duration = match media_data_cache.get(&file_hash) {
    Some(meta) => meta.duration_seconds,
    None => {
      let inspection_raw_data =
        inspect::ffprobe_runner::inspect_file(local_path, video_inspect_timeout).await?;
      let media_meta_data = inspect::ffprobe_mapper::map_media_meta(inspection_raw_data)?;
      media_data_cache.insert(file_hash, media_meta_data.clone());
      media_meta_data.duration_seconds
    }
  };

  if duration <= 0.0 {
    Err(InputError::DataError("File has zero duration".to_string()))?;
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
  async fn test_read_plain_file_name_success() {
    let content = b"fake video bytes";
    let file_name: &str = "video.mp4";
    let mut multipart = multipart_from_field(Some(file_name), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let form_data_meta = read_form_data_meta(&mut field).await.unwrap();

    assert_eq!(form_data_meta.original_file_name, file_name);
    assert_eq!(form_data_meta.safe_file_name, file_name);
  }

  #[tokio::test]
  async fn test_strips_path_traversal_in_file_name_success() {
    let content = b"fake video bytes";
    let file_name: &str = "../../video.mp4";
    let mut multipart = multipart_from_field(Some(file_name), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let form_data_meta = read_form_data_meta(&mut field).await.unwrap();

    assert_eq!(form_data_meta.original_file_name, file_name);
    assert_eq!(form_data_meta.safe_file_name, file_name[6..].to_string());
  }

  #[tokio::test]
  async fn test_read_missing_file_name_failure() {
    let content = b"fake video bytes";
    let mut multipart = multipart_from_field(None, content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let form_data_meta = read_form_data_meta(&mut field).await;
    assert!(form_data_meta.is_err());
    let err = form_data_meta.unwrap_err();

    let _server_error =
      ServerError::InputError(InputError::DataError("Missing file name".to_string()));
    assert!(matches!(err, _server_error));
  }

  #[tokio::test]
  async fn test_read_missing_file_extension_failure() {
    let content = b"fake video bytes";
    let file_name: &str = "video";
    let mut multipart = multipart_from_field(Some(file_name), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let form_data_meta = read_form_data_meta(&mut field).await;
    assert!(form_data_meta.is_err());
    let err = form_data_meta.unwrap_err();

    let _server_error = ServerError::InputError(InputError::DataError(format!(
      "Missing file extension {file_name}"
    )));
    assert!(matches!(err, _server_error));
  }

  #[tokio::test]
  async fn test_read_unsupported_file_extension_failure() {
    let content = b"fake video bytes";
    let file_name: &str = "video.mp3";
    let mut multipart = multipart_from_field(Some(file_name), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();

    let form_data_meta = read_form_data_meta(&mut field).await;
    assert!(form_data_meta.is_err());
    let err = form_data_meta.unwrap_err();

    let _server_error = ServerError::InputError(InputError::DataError(
      "File extension is not supported: mp3".to_string(),
    ));
    assert!(matches!(err, _server_error));
  }

  #[tokio::test]
  async fn test_read_video_to_file_success() {
    let temp_dir = tempdir().unwrap();
    let content = b"fake video bytes";
    let file_name: &str = "video.mp4";
    let mut multipart = multipart_from_field(Some(file_name), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();
    let form_data_meta = ReadFormDataMeta {
      original_file_name: file_name.to_string(),
      safe_file_name: file_name.to_string(),
    };

    let local_path = read_form_data_to_file(&mut field, &form_data_meta, temp_dir.path())
      .await
      .unwrap();

    assert!(Path::new(&local_path).starts_with(temp_dir.path()));
    let saved = tokio::fs::read(&local_path).await.unwrap();
    assert_eq!(saved, content);
  }

  #[tokio::test]
  async fn test_read_file_bytes_fail() {
    let temp_dir = tempdir().unwrap();
    let content = b"";
    let file_name: &str = "video.mp4";
    let mut multipart = multipart_from_field(Some(file_name), content).await;
    let mut field = multipart.next_field().await.unwrap().unwrap();
    let form_data_meta = ReadFormDataMeta {
      original_file_name: file_name.to_string(),
      safe_file_name: file_name.to_string(),
    };

    let local_path_result =
      read_form_data_to_file(&mut field, &form_data_meta, temp_dir.path()).await;
    assert!(local_path_result.is_err());
    let err = local_path_result.unwrap_err();

    let _server_error =
      ServerError::InputError(InputError::DataError("Form data is empty".to_string()));
    assert!(matches!(err, _server_error));
  }
}
