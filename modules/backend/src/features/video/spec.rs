use utoipa::ToSchema;
use utoipa::openapi::{ObjectBuilder, Schema, Type};

use crate::features::video::process::configs::{codec, extension, operation};

#[allow(unused)]
#[derive(ToSchema)]
pub struct InspectVideoPayload {
  #[schema(value_type = String, format = Binary)]
  pub video: Vec<u8>,
}

// Allowed values come from `process::configs`, the same constants the form reader parses,
// so the spec and the validation stay in sync.
#[allow(unused)]
#[derive(ToSchema)]
pub struct ProcessVideoPayload {
  #[schema(schema_with = operation_schema)]
  pub operation: String,
  #[schema(schema_with = codec_schema)]
  pub codec: String,
  #[schema(schema_with = preset_schema)]
  pub preset: String,
  #[schema(schema_with = quality_schema)]
  pub quality: String,
  #[schema(schema_with = format_schema)]
  pub format: String,
  #[schema(value_type = String, format = Binary)]
  pub video: Vec<u8>,
}

// Binary response body for openapi schema typings
#[allow(unused)]
#[derive(ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct ProcessedVideoFile(Vec<u8>);

fn string_enum_schema(description: &str, values: &[&str]) -> Schema {
  ObjectBuilder::new()
    .schema_type(Type::String)
    .description(Some(description))
    .enum_values(Some(values.iter().copied()))
    .into()
}

fn operation_schema() -> Schema {
  string_enum_schema("Processing operation", &[operation::COMPRESS])
}

fn codec_schema() -> Schema {
  string_enum_schema("Video codec", &[codec::x264::TYPE, codec::vp9::TYPE])
}

fn preset_schema() -> Schema {
  string_enum_schema(
    "Encoder preset. Must be sent after the `codec` field. Required for every codec, but only \
     libx264 validates and uses it; libvpx-vp9 ignores the value.",
    &[
      codec::x264::preset::FAST,
      codec::x264::preset::BALANCED,
      codec::x264::preset::BEST_COMPRESSION,
    ],
  )
}

fn format_schema() -> Schema {
  string_enum_schema(
    "Output container. Sets the extension and content type of the returned file. webm supports \
     libvpx-vp9 only.",
    &[
      extension::MP4,
      extension::MKV,
      extension::WEBM,
      extension::MOV,
    ],
  )
}

fn quality_schema() -> Schema {
  string_enum_schema(
    "Constant rate factor (CRF), lower is higher quality. Must be sent after the `codec` field. \
     libx264: 18 (high), 23 (standard), 28 (low); libvpx-vp9: 24 (high), 31 (standard), 38 (low).",
    &[
      codec::x264::quality::HIGH,
      codec::x264::quality::STANDARD,
      codec::x264::quality::LOW,
      codec::vp9::quality::HIGH,
      codec::vp9::quality::STANDARD,
      codec::vp9::quality::LOW,
    ],
  )
}
