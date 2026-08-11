use std::fs::File;
use std::io::Write;

use backend::core::openapi::OpenApiSpec;
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let spec = OpenApiSpec::openapi().to_pretty_json()?;
  let mut file = File::create("openapi.json")?;
  file.write_all(spec.as_bytes())?;

  Ok(())
}
