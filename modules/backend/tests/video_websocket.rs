mod common;
use common::utils;

#[cfg(test)]
mod test_video_websocket {
  use axum::body::Bytes;
  use axum_test::{TestServer, WsMessage};
  use sqlx::PgPool;
  use video_processing_server::core::error::ApplicationError;
  use video_processing_server::router::routes;

  use crate::utils::{get_session_cookie_jar, setup_router, with_base_route};

  #[sqlx::test()]
  async fn test_ping_pong(pool: PgPool) -> Result<(), ApplicationError> {
    let router = setup_router(pool).await?;
    let server = TestServer::builder().http_transport().build(router);

    let mut client = server
      .get_websocket(&with_base_route(routes::VIDEO_WEB_SOCKET))
      .add_cookies(get_session_cookie_jar())
      .await
      .into_websocket()
      .await;

    client.send_message(WsMessage::Ping(Bytes::new())).await;
    let response = client.receive_message().await;
    assert!(response.is_pong());

    Ok(())
  }

  #[sqlx::test()]
  async fn test_duplicate_connection(pool: PgPool) -> Result<(), ApplicationError> {
    let router = setup_router(pool).await?;
    let server = TestServer::builder().http_transport().build(router);
    let cookie = get_session_cookie_jar();

    let _client_1 = server
      .get_websocket(&with_base_route(routes::VIDEO_WEB_SOCKET))
      .add_cookies(cookie.clone())
      .await;

    let mut client_2 = server
      .get_websocket(&with_base_route(routes::VIDEO_WEB_SOCKET))
      .add_cookies(cookie.clone())
      .await
      .into_websocket()
      .await;

    let response = client_2.receive_message().await;
    assert!(response.is_close());

    Ok(())
  }

  #[sqlx::test()]
  async fn test_reconnect(pool: PgPool) -> Result<(), ApplicationError> {
    let router = setup_router(pool).await?;
    let server = TestServer::builder().http_transport().build(router);
    let cookie = get_session_cookie_jar();

    let connection = server
      .get_websocket(&with_base_route(routes::VIDEO_WEB_SOCKET))
      .add_cookies(cookie.clone())
      .await;
    connection.assert_status_switching_protocols();

    let client = connection.into_websocket().await;
    client.close().await;

    let new_connection = server
      .get_websocket(&with_base_route(routes::VIDEO_WEB_SOCKET))
      .add_cookies(cookie.clone())
      .await;
    new_connection.assert_status_switching_protocols();

    Ok(())
  }
}
