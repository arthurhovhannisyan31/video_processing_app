#[cfg(test)]
#[allow(dead_code)]
pub const MOCK_SESSION_ID: &str = "e2ba73db-2805-4ded-91f8-50193e38dcb4";

/// Valid processing arguments. Order matters: 'codec' must be sent before 'preset'.
pub const DEFAULT_FORM_DATA_ARGS: [(&str, &str); 4] = [
  ("operation", "compress"),
  ("codec", "libx264"),
  ("preset", "medium"),
  ("quality", "23"),
];
