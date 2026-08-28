use std::time::Duration;

pub const VIDEO_MAX_BODY_SIZE: usize = 100 * 1024 * 1024;
pub const VIDEO_API_INSPECT_TIMEOUT: Duration = Duration::from_secs(20);
pub const VIDEO_API_PROCESS_TIMEOUT: Duration = Duration::from_secs(300);
pub const VIDEO_RATE_LIMIT_PERIOD: u16 = 3600;
pub const VIDEO_RATE_LIMIT_SIZE: u16 = 100;
