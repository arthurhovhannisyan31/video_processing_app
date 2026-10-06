use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use governor::middleware::NoOpMiddleware;
use parking_lot::Mutex;
use tower_governor::governor::{
  DEFAULT_BURST_SIZE, DEFAULT_PERIOD, GovernorConfig, GovernorConfigBuilder,
};
use tower_governor::key_extractor::KeyExtractor;

use crate::core::error::ServerError;
use crate::features::system::state::RateLimiters;

const GOVERNOR_CLEANUP_INTERVAL: u64 = 300;

pub fn build_rate_limiter_config<K: KeyExtractor>(
  key_extractor: K,
  rate_limit_period: Option<Duration>,
  rate_limit_size: Option<u32>,
) -> Result<GovernorConfig<K, NoOpMiddleware>, ServerError> {
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

pub fn rate_limiters_cleanup(rate_limiters: Arc<Mutex<RateLimiters>>) {
  let mut counter = 0;
  let interval = Duration::from_secs(GOVERNOR_CLEANUP_INTERVAL);

  std::thread::spawn(move || {
    loop {
      std::thread::sleep(interval);
      counter += 1;
      {
        let rate_limiters = rate_limiters.lock();
        for rate_limiter in (*rate_limiters).iter() {
          rate_limiter.retain_recent();
          // Reallocate rate limiter hash map every 12 cycles
          if counter >= 10 {
            rate_limiter.shrink_to_fit();
            counter = 0;
          }
        }
      }
    }
  });
}
