pub const VIDEO_MAX_BODY_SIZE: usize = 100 * 1024 * 1024;
pub const VIDEO_INSPECT_TIMEOUT: u32 = 20;
pub const VIDEO_PROCESS_TIMEOUT: u32 = 300;
// # When quota exceeded replenish 1 more element each 60 seconds
pub const VIDEO_RATE_LIMIT_PERIOD: u64 = 60;
pub const VIDEO_RATE_LIMIT_SIZE: u32 = 100;
pub const VIDEO_MIN_PROGRESS_VALUE: f64 = 0.0;
pub const VIDEO_MAX_PROGRESS_VALUE: f64 = 1.0;
pub static FILE_EXTENSIONS_WHITE_LIST: &[&str] = &["mp4"];
