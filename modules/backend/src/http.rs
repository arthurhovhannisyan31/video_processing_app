use axum_session::SessionStore;
use axum_session_sqlx::SessionPgPool;
use tokio::net::TcpListener;

use crate::core::app_state::AppState;
use crate::core::error::ServerError;
use crate::router::build_router;

pub async fn init_http_server(
  app_state: AppState,
  session_store: SessionStore<SessionPgPool>,
) -> Result<(), ServerError> {
  let socket_addr = app_state.app_config.socket_addr;

  let root_router = build_router(app_state, session_store)?;
  let listener = TcpListener::bind(socket_addr).await?;

  Ok(axum::serve(listener, root_router).await?)
}
