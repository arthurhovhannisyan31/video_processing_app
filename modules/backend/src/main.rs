mod core;
mod features;
mod http;
mod router;

use core::app_config::AppConfig;
use core::app_state::AppState;
use core::database::{create_pool, run_migrations};
use core::error::ServerError;
use core::logging::init_logging;

use http::init_http_server;

use crate::core::governor::rate_limiters_cleanup;

fn main() -> Result<(), ServerError> {
  init_logging()?;

  let app_config = AppConfig::from_env()?;

  let _guard = sentry::init(
    sentry::ClientOptions::new()
      .dsn(&app_config.sentry_dsn)
      .maybe_release(sentry::release_name!())
      .send_default_pii(false),
  );

  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?
    .block_on(async {
      let pool = create_pool(&app_config.database_url, app_config.db_max_connections).await?;

      run_migrations(&pool).await?;

      let app_state = AppState::new(app_config, pool);
      rate_limiters_cleanup(app_state.system_state.rate_limiters.clone());

      init_http_server(app_state).await.inspect_err(|err| {
        sentry::capture_error(&err);
      })?;

      Ok::<(), ServerError>(())
    })
    .inspect_err(|err| {
      sentry::capture_error(&err);
    })?;

  Ok(())
}
