mod utils;

#[cfg(test)]
mod test_video_websocket {
  use axum::body::Bytes;
  use axum_test::{TestServer, WsMessage};
  use sqlx::PgPool;
  use uuid::Uuid;
  use video_processing_server::core::error::ApplicationError;
  use video_processing_server::router::routes;

  use crate::utils::{setup_router, with_base_route};

  #[sqlx::test()]
  async fn test_ping_pong(pool: PgPool) -> Result<(), ApplicationError> {
    let router = setup_router(pool)?;
    let server = TestServer::builder().http_transport().build(router);

    let mut client = server
      .get_websocket(&with_base_route(&format!(
        "{}/{}",
        routes::VIDEO_WEB_SOCKET,
        Uuid::new_v4()
      )))
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
    let router = setup_router(pool)?;
    let server = TestServer::builder().http_transport().build(router);
    let user_id = Uuid::new_v4();

    let _client_1 = server
      .get_websocket(&with_base_route(&format!(
        "{}/{}",
        routes::VIDEO_WEB_SOCKET,
        user_id
      )))
      .await;

    let mut client_2 = server
      .get_websocket(&with_base_route(&format!(
        "{}/{}",
        routes::VIDEO_WEB_SOCKET,
        user_id
      )))
      .await
      .into_websocket()
      .await;

    let response = client_2.receive_message().await;
    assert!(response.is_close());

    Ok(())
  }

  #[sqlx::test()]
  async fn test_reconnect(pool: PgPool) -> Result<(), ApplicationError> {
    let router = setup_router(pool)?;
    let server = TestServer::builder().http_transport().build(router);
    let user_id = Uuid::new_v4();

    let connection = server
      .get_websocket(&with_base_route(&format!(
        "{}/{}",
        routes::VIDEO_WEB_SOCKET,
        user_id
      )))
      .await;
    connection.assert_status_switching_protocols();

    let client = connection.into_websocket().await;
    client.close().await;

    let new_connection = server
      .get_websocket(&with_base_route(&format!(
        "{}/{}",
        routes::VIDEO_WEB_SOCKET,
        user_id
      )))
      .await;
    new_connection.assert_status_switching_protocols();

    Ok(())
  }
}
