use std::env;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::FromRef;
use serde::Deserialize;
use tracing::warn;

use crate::core::app_state::AppState;
use crate::core::error::{ServerError, StartupError};
use crate::features::auth::constants::{
  PASSWORD_RESET_TTL_MIN, SMTP_DEV_FROM, SMTP_DEV_HOST, SMTP_DEV_PORT,
};
use crate::features::video::constants::{
  VIDEO_INSPECT_TIMEOUT, VIDEO_MAX_BODY_SIZE, VIDEO_PROCESS_TIMEOUT, VIDEO_RATE_LIMIT_PERIOD,
  VIDEO_RATE_LIMIT_SIZE,
};

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
  pub host: String,
  pub http_port: u16,
  pub database_url: String,
  pub jwt_secret: String,
  pub cors_origins: Vec<String>,
  pub db_max_connections: u32,
  pub is_production: bool,
  pub video: VideoConfig,
  pub mock_password_hash: String,
  pub sentry_dsn: String,
  pub smtp: SmtpConfig,
  pub frontend_url: String,
  pub password_reset_ttl: Duration,
}

impl AppConfig {
  pub fn from_env() -> Result<Self, ServerError> {
    let is_container = env::var("CONTAINER")
      .unwrap_or("false".to_owned())
      .eq("true");
    // Load variables when run locally
    if !is_container {
      dotenvy::dotenv().map_err(StartupError::Dotenv)?;
    }

    let host = env::var("BACKEND_HOST").unwrap_or("localhost".into());
    let http_port = env::var("BACKEND_HTTP_PORT")
      .unwrap_or("8080".to_string())
      .parse()
      .map_err(|e| StartupError::VarError(format!("Invalid BACKEND_HTTP_PORT variable: {e}")))?;
    let database_url = env::var("DATABASE_URL")
      .map_err(|e| StartupError::VarError(format!("Missing DATABASE_URL: {e}")))?;
    let jwt_secret = env::var("BACKEND_JWT_SECRET")
      .map_err(|e| StartupError::VarError(format!("Missing BACKEND_JWT_SECRET: {e}")))?;
    let cors_origins = env::var("BACKEND_CORS_ORIGINS")
      .map_err(|e| StartupError::VarError(format!("Missing BACKEND_CORS_ORIGINS: {e}")))?
      .split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();
    let db_max_connections = env::var("BACKEND_DB_MAX_CONNECTIONS")
      .unwrap_or("10".to_string())
      .parse::<u32>()
      .map_err(|e| {
        StartupError::VarError(format!("Failed parsing BACKEND_DB_MAX_CONNECTIONS: {e}"))
      })?;
    let is_production = env::var("IS_PRODUCTION")
      .unwrap_or("false".to_string())
      .eq("true");
    let mock_password_hash = env::var("BACKEND_VIDEO_MOCK_PASSWORD_HASH").map_err(|e| {
      StartupError::VarError(format!("Missing BACKEND_VIDEO_MOCK_PASSWORD_HASH: {e}"))
    })?;
    let sentry_dsn = env::var("SENTRY_DSN")
      .map_err(|e| StartupError::VarError(format!("Missing SENTRY_DSN: {e}")))?;
    let smtp = SmtpConfig::from_env(is_production)?;
    let frontend_url = env_or_warn_in_production("BACKEND_FRONTEND_URL", is_production)?
      .unwrap_or("http://localhost:3000".to_string())
      .trim_end_matches('/')
      .to_string();
    let password_reset_ttl = env::var("BACKEND_PASSWORD_RESET_TTL_MIN")
      .unwrap_or(PASSWORD_RESET_TTL_MIN.to_string())
      .parse::<u64>()?;
    let video = VideoConfig::from_env()?;

    Ok(Self {
      host,
      http_port,
      database_url,
      jwt_secret,
      cors_origins,
      db_max_connections,
      is_production,
      mock_password_hash,
      sentry_dsn,
      smtp,
      video,
      frontend_url,
      password_reset_ttl: Duration::from_secs(password_reset_ttl * 60),
    })
  }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SmtpConfig {
  pub host: String,
  pub port: u16,
  pub username: Option<String>,
  pub password: Option<String>,
  pub from: String,
  pub tls: bool,
}

impl SmtpConfig {
  /// Local development falls back to an unauthenticated plain SMTP server (e.g. Mailpit)
  pub fn from_env(is_production: bool) -> Result<Self, ServerError> {
    let host = env_or_warn_in_production("BACKEND_SMTP_HOST", is_production)?;
    let tls = host.is_some();
    let host = host.unwrap_or(SMTP_DEV_HOST.to_string());
    let port = match env::var("BACKEND_SMTP_PORT") {
      Ok(port) => port
        .parse::<u16>()
        .map_err(|e| StartupError::VarError(format!("Invalid BACKEND_SMTP_PORT variable: {e}")))?,
      Err(_) if tls => lettre::transport::smtp::SUBMISSIONS_PORT,
      Err(_) => SMTP_DEV_PORT,
    };
    let username = env::var("BACKEND_SMTP_USERNAME")
      .ok()
      .filter(|s| !s.is_empty());
    let password = env::var("BACKEND_SMTP_PASSWORD")
      .ok()
      .filter(|s| !s.is_empty());
    let from = env_or_warn_in_production("BACKEND_SMTP_FROM", is_production)?
      .unwrap_or(SMTP_DEV_FROM.to_string());

    Ok(SmtpConfig {
      host,
      port,
      username,
      password,
      from,
      tls,
    })
  }
}

fn env_or_warn_in_production(
  name: &str,
  is_production: bool,
) -> Result<Option<String>, ServerError> {
  let value = env::var(name).ok().filter(|s| !s.is_empty());
  if value.is_none() && is_production {
    warn!("{name} is not set, falling back to local development value");
  }

  Ok(value)
}

#[derive(Clone, Debug, Deserialize)]
pub struct VideoConfig {
  pub max_body_size: usize,
  pub rate_limit_period: u64,
  pub rate_limit_size: u32,
  pub inspect_timeout: Duration,
  pub process_timeout: Duration,
}

impl VideoConfig {
  pub fn from_env() -> Result<Self, ServerError> {
    let is_container = env::var("CONTAINER")
      .unwrap_or("false".to_owned())
      .eq("true");
    // Load variables when run locally
    if !is_container {
      dotenvy::dotenv().map_err(StartupError::Dotenv)?;
    }

    let max_body_size = env::var("BACKEND_VIDEO_MAX_BODY_SIZE")
      .unwrap_or(VIDEO_MAX_BODY_SIZE.to_string())
      .parse::<usize>()?;
    let rate_limit_period = env::var("BACKEND_VIDEO_RATE_LIMIT_PERIOD_SEC")
      .unwrap_or(VIDEO_RATE_LIMIT_PERIOD.to_string())
      .parse::<u64>()?;
    let rate_limit_size = env::var("BACKEND_VIDEO_RATE_LIMIT_SIZE")
      .unwrap_or(VIDEO_RATE_LIMIT_SIZE.to_string())
      .parse::<u32>()?;
    let inspect_timeout = env::var("BACKEND_VIDEO_INSPECT_TIMEOUT")
      .unwrap_or(VIDEO_INSPECT_TIMEOUT.to_string())
      .parse::<u64>()?;
    let process_timeout = env::var("BACKEND_VIDEO_PROCESS_TIMEOUT")
      .unwrap_or(VIDEO_PROCESS_TIMEOUT.to_string())
      .parse::<u64>()?;

    Ok(Self {
      max_body_size,
      rate_limit_period,
      rate_limit_size,
      inspect_timeout: Duration::from_secs(inspect_timeout),
      process_timeout: Duration::from_secs(process_timeout),
    })
  }
}

impl FromRef<AppState> for Arc<AppConfig> {
  fn from_ref(app_state: &AppState) -> Self {
    app_state.app_config.clone()
  }
}
