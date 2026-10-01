use std::time::Duration;

use mini_moka::sync::Cache;

use crate::features::video::model::MediaMetadata;

// Time to live, 1 hour
const CACHE_TTL: u64 = 3600;
// Time to idle, 10 minutes
const CACHE_TTI: u64 = 600;

pub type MediaDataCache = Cache<u64, MediaMetadata>;

pub fn build_media_data_cache() -> MediaDataCache {
  Cache::<u64, MediaMetadata>::builder()
    .time_to_live(Duration::from_secs(CACHE_TTL))
    .time_to_idle(Duration::from_secs(CACHE_TTI))
    .build()
}
