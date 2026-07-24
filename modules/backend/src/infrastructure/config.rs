use crate::infrastructure::error::ServerError;

use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
  pub host: String,
  pub http_port: u16,
  pub database_url: String,
  pub jwt_secret: String,
  pub cors_origins: Vec<String>,
  pub db_max_connections: u32,
  pub is_production: bool,
}

impl AppConfig {
  pub fn from_env() -> Result<Self, ServerError> {
    let is_container = env::var("CONTAINER")
      .unwrap_or("false".to_owned())
      .eq("true");
    // Load variables when run locally
    if !is_container {
      dotenvy::dotenv()?;
    }

    let host = env::var("BACKEND_HOST").unwrap_or("localhost".into());
    let http_port = env::var("BACKEND_HTTP_PORT")
      .unwrap_or("8080".to_string())
      .parse()
      .map_err(|e| {
        ServerError::VarError(format!(
          "Invalid BACKEND_HTTP_PORT variable: {e}"
        ))
      })?;
    let database_url = env::var("DATABASE_URL").map_err(|e| {
      ServerError::VarError(format!("Missing DATABASE_URL: {e}"))
    })?;
    let jwt_secret = env::var("BACKEND_JWT_SECRET").map_err(|e| {
      ServerError::VarError(format!("Missing BACKEND_JWT_SECRET: {e}"))
    })?;
    let cors_origins = env::var("BACKEND_CORS_ORIGINS")
      .map_err(|e| {
        ServerError::VarError(format!("Missing BACKEND_CORS_ORIGINS: {e}"))
      })?
      .split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();
    let db_max_connections = env::var("BACKEND_DB_MAX_CONNECTIONS")
      .unwrap_or("10".to_string())
      .parse::<u32>()
      .map_err(|e| {
        ServerError::VarError(format!(
          "Failed parsing BACKEND_DB_MAX_CONNECTIONS: {e}"
        ))
      })?;
    let is_production = env::var("IS_PRODUCTION")
      .unwrap_or("false".to_string())
      .eq("true");

    Ok(Self {
      host,
      http_port,
      database_url,
      jwt_secret,
      cors_origins,
      db_max_connections,
      is_production,
    })
  }
}
