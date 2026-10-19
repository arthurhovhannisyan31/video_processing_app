use std::path::Path;
use std::str::FromStr;

use axum::extract::Multipart;

use crate::core::error::{InputError, ServerError};
use crate::features::video::configs::FieldName;
use crate::features::video::helpers::{read_form_data_meta, read_form_data_to_file};
use crate::features::video::process::types::{
  Codec, Operation, Preset, ProcessVideoMeta, Quality, VideoFormat,
};

pub async fn read(
  mut media_data: Multipart,
  temp_dir: &Path,
) -> Result<ProcessVideoMeta, ServerError> {
  let mut file_name = String::new();
  let mut local_path = String::new();
  let mut operation: Option<Operation> = None;
  let mut codec: Option<Codec> = None;
  let mut preset: Option<Preset> = None;
  let mut quality: Option<Quality> = None;
  let mut format: Option<VideoFormat> = None;

  while let Some(mut field) = media_data
    .next_field()
    .await
    .map_err(InputError::Multipart)?
  {
    let field_name = field
      .name()
      .ok_or(InputError::DataError("Missing field name".to_string()))?
      .to_string();

    let field_name = FieldName::from_str(&field_name)?;

    match field_name {
      FieldName::Video => {
        let form_data_meta = read_form_data_meta(&mut field).await?;
        local_path = read_form_data_to_file(&mut field, &form_data_meta, temp_dir).await?;
        file_name = form_data_meta.original_file_name;
      }
      FieldName::Operation => {
        let str = field.text().await.map_err(InputError::Multipart)?;
        operation = Some(Operation::from_str(&str)?);
      }
      FieldName::Codec => {
        let str = field.text().await.map_err(InputError::Multipart)?;
        codec = Some(Codec::from_str(&str)?);
      }
      FieldName::Preset => {
        let str = field.text().await.map_err(InputError::Multipart)?;
        match &codec {
          Some(codec) => {
            preset = Some(Preset::get_by_codec(&str, codec)?);
          }
          None => {
            Err(InputError::DataError(
              "Missing 'codec' field, ensure it is listed before 'preset' field".to_string(),
            ))?;
          }
        }
      }
      FieldName::Quality => {
        let str = field.text().await.map_err(InputError::Multipart)?;
        quality = Some(Quality::from_str(&str)?);
      }
      FieldName::Format => {
        let str = field.text().await.map_err(InputError::Multipart)?;
        format = Some(VideoFormat::from_str(&str)?);
      }
    }
  }

  if file_name.is_empty() || local_path.is_empty() {
    Err(InputError::DataError("Missing 'video' field".to_string()))?;
  };
  let Some(operation) = operation else {
    return Err(InputError::DataError(
      "Missing 'operation' field".to_string(),
    ))?;
  };
  let Some(codec) = codec else {
    return Err(InputError::DataError("Missing 'codec' field".to_string()))?;
  };
  let Some(preset) = preset else {
    return Err(InputError::DataError("Missing 'preset' field".to_string()))?;
  };
  let Some(quality) = quality else {
    return Err(InputError::DataError("Missing 'quality' field".to_string()))?;
  };
  let Some(format) = format else {
    return Err(InputError::DataError("Missing 'format' field".to_string()))?;
  };

  Ok(ProcessVideoMeta {
    codec,
    file_name,
    local_path,
    operation,
    preset,
    quality,
    format,
  })
}
