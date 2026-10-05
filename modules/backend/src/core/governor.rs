use std::time::Duration;

use anyhow::anyhow;
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::{
  DEFAULT_BURST_SIZE, DEFAULT_PERIOD, GovernorConfig, GovernorConfigBuilder,
};
use tower_governor::key_extractor::{KeyExtractor, SmartIpKeyExtractor};

use crate::core::error::ServerError;

pub fn build_governor_config(
  key_extractor: impl KeyExtractor,
  rate_limit_period: Option<Duration>,
  rate_limit_size: Option<u32>,
) -> Result<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>, ServerError> {
  let rate_limit_period = rate_limit_period.unwrap_or(DEFAULT_PERIOD);
  let rate_limit_size = rate_limit_size.unwrap_or(DEFAULT_BURST_SIZE);

  GovernorConfigBuilder::default()
    .period(rate_limit_period)
    .burst_size(rate_limit_size)
    .key_extractor(key_extractor)
    .finish()
    .ok_or(ServerError::OtherError(anyhow!(
      "Wrong tower_governor configuration"
    )))
}
