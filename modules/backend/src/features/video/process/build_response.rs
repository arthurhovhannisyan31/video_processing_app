use std::path::Path;

use axum::body::Body;
use axum::http::{Response, StatusCode, header};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::core::error::ServerError;

pub async fn build_response(
  file_path: &str,
  output_path: &str,
) -> Result<Response<Body>, ServerError> {
  let file = File::open(output_path).await.map_err(ServerError::IO)?;
  let stream = ReaderStream::new(file);
  let original_name = Path::new(&file_path)
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or("video.mp4");

  let response = Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, "video/mp4")
    .header(
      header::CONTENT_DISPOSITION,
      format!("attachment; filename=\"{}\"", original_name),
    )
    .body(Body::from_stream(stream))
    .map_err(ServerError::HttpError)?;

  Ok(response)
}
