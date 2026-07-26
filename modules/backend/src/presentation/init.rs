use crate::{
  infrastructure::{cors::build_cors_layer, error::ServerError},
  presentation::{
    auth::get_auth_router, protected::get_protected_router, state::AppState,
    utilities::get_utilities_router,
  },
};

use axum::Router;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub fn build_router(app_state: AppState) -> Router {
  let merged_router = Router::new()
    .merge(get_auth_router())
    .merge(get_utilities_router())
    .merge(get_protected_router(app_state.clone()))
    .with_state(app_state.clone());

  Router::new()
    .nest("/api", merged_router)
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new())
    .layer(build_cors_layer(app_state.app_config.clone()))
}

pub async fn init_http_server(app_state: AppState) -> Result<(), ServerError> {
  let host = app_state.app_config.host.to_string();
  let http_port = app_state.app_config.http_port;

  let root_router = build_router(app_state);
  let listener = TcpListener::bind((host, http_port)).await?;

  Ok(axum::serve(listener, root_router).await?)
}
