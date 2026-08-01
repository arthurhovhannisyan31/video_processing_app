use crate::core::error::ApplicationError;
use crate::features::video::configs::ffprobe::FfprobeType;
use crate::features::video::ffprobe_mapper::ffprobe_mapper;
use crate::features::video::ffprobe_runner::ffprobe_runner;

use axum::extract::Multipart;

pub async fn read_video(
  mut media_data: Multipart,
) -> Result<FfprobeType, ApplicationError> {
  while let Some(field) = media_data.next_field().await? {
    let file_name = field
      .file_name()
      .ok_or(ApplicationError::BadRequest(
        "Missing file_name".to_string(),
      ))?
      .to_string();

    if field.name() == Some("video") {
      let ffprobe_output = ffprobe_runner(field).await?;
      println!("json_data: {ffprobe_output:#?}");
      let mut ffprobe_mapped_data = ffprobe_mapper(ffprobe_output)?;

      if let Some(format) = ffprobe_mapped_data.format.as_mut() {
        format.filename = file_name;
      }

      return Ok(ffprobe_mapped_data);
    }
  }

  Err(ApplicationError::BadRequest(
    "Missing 'video' field".to_string(),
  ))
}
