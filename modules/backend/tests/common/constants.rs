#[cfg(test)]
#[allow(dead_code)]
pub const MOCK_SESSION_ID: &str = "e2ba73db-2805-4ded-91f8-50193e38dcb4";

/// Valid processing arguments. Order matters: 'codec' must be sent before 'preset' and 'quality'.
#[allow(dead_code)]
pub const DEFAULT_FORM_DATA_ARGS: [(&str, &str); 5] = [
  ("operation", "compress"),
  ("codec", "libx264"),
  ("preset", "medium"),
  ("quality", "23"),
  ("format", "mp4"),
];

/// Valid VP9 processing arguments. VP9 ignores 'preset', but the field is still required.
#[allow(dead_code)]
pub const VP9_FORM_DATA_ARGS: [(&str, &str); 5] = [
  ("operation", "compress"),
  ("codec", "libvpx-vp9"),
  ("preset", "medium"),
  ("quality", "31"),
  ("format", "webm"),
];
