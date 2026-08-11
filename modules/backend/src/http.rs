use tokio::net::TcpListener;

use crate::core::app_state::AppState;
use crate::core::error::ServerError;
use crate::router::build_router;

pub async fn init_http_server(app_state: AppState) -> Result<(), ServerError> {
  let host = app_state.app_config.host.to_string();
  let http_port = app_state.app_config.http_port;

  let root_router = build_router(app_state)?;
  let listener = TcpListener::bind((host, http_port)).await?;

  Ok(axum::serve(listener, root_router).await?)
}
