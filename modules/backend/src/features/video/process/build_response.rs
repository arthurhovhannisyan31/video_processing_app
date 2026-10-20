use axum::body::Body;
use axum::http::{Response, StatusCode, header};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::core::error::ServerError;
use crate::features::video::process::types::OutputVideoMeta;

pub async fn build_response(
  output_meta: &OutputVideoMeta,
  mime_type: &str,
) -> Result<Response<Body>, ServerError> {
  let file = File::open(&output_meta.path)
    .await
    .map_err(ServerError::IO)?;
  let stream = ReaderStream::new(file);

  let response = Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, mime_type)
    .header(
      header::CONTENT_DISPOSITION,
      format!("attachment; filename=\"{}\"", output_meta.name),
    )
    .body(Body::from_stream(stream))?;

  Ok(response)
}
