use sentry::integrations::tracing::LogLayer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt, registry};

use crate::core::error::ServerError;

pub fn init_logging() -> Result<(), ServerError> {
  let filter = EnvFilter::try_from_default_env()
    .or_else(|_| EnvFilter::try_new("info,video_processing_server=info,tower_http=trace"))?;

  let fmt_layer = fmt::layer()
    .with_target(false)
    .with_level(true)
    .with_timer(fmt::time::UtcTime::rfc_3339());

  registry()
    .with(filter)
    .with(fmt_layer)
    .with(LogLayer::new().with_filter(LevelFilter::INFO))
    .init();

  Ok(())
}
