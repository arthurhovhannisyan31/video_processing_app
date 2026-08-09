mod core;
mod features;
mod http;
mod router;

use core::app_config::AppConfig;
use core::app_state::{AppState, AuthState};
use core::database::{create_pool, run_migrations};
use core::error::ServerError;
use core::jwt::JwtService;
use core::logging::init_logging;
use std::sync::Arc;

use features::auth::repository::PostgresUserRepository;
use features::auth::service::AuthService;
use http::init_http_server;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
  init_logging();

  let app_config = AppConfig::from_env()?;
  let pool = create_pool(&app_config.database_url, app_config.db_max_connections).await?;

  run_migrations(&pool).await?;

  let jwt_service = JwtService::new(app_config.jwt_secret.clone());
  let users_repo = PostgresUserRepository::new(pool.clone());
  let auth_service = AuthService::new(users_repo, jwt_service.clone());
  let app_state = AppState {
    auth_state: Arc::new(AuthState {
      auth_service,
      jwt_service,
    }),
    app_config: Arc::new(app_config),
  };

  init_http_server(app_state).await?;

  Ok(())
}
