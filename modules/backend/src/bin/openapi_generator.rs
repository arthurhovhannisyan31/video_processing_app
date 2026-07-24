use backend::infrastructure::openapi::OpenApiSpec;
use std::fs::File;
use std::io::Write;
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let spec = OpenApiSpec::openapi().to_pretty_json()?;

  let mut file = File::create("openapi.json")?;
  file.write_all(spec.as_bytes())?;

  println!("Successfully generated openapi.json");
  Ok(())
}
