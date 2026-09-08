use futures::StreamExt;

use super::{ConnectionManager, RealtimeMessage};

/// Start a Redis pubsub listener that forwards messages to local ConnectionManager.
/// Enables cross-instance fanout when running multiple server instances.
pub async fn start_redis_pubsub(
    redis: redis::Client,
    manager: ConnectionManager,
) -> Result<(), redis::RedisError> {
    let mut pubsub_conn = redis.get_async_pubsub().await?;
    pubsub_conn.subscribe("ficnexus:rt").await?;

    // into_on_message() consumes self, returning an owned stream.
    let mut stream = pubsub_conn.into_on_message();

    tokio::spawn(async move {
        while let Some(msg) = stream.next().await {
            if let Ok(payload) = msg.get_payload::<String>() {
                if let Ok(realtime_msg) = serde_json::from_str::<RealtimeMessage>(&payload) {
                    manager.publish(&realtime_msg).await;
                }
            }
        }
    });

    Ok(())
}

/// Publish a message to Redis for cross-instance fanout.
pub async fn publish_to_redis(
    redis: &redis::Client,
    msg: &RealtimeMessage,
) -> Result<(), redis::RedisError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let payload = serde_json::to_string(msg)
        .map_err(|e| redis::RedisError::from((redis::ErrorKind::Io, "serialize", e.to_string())))?;
    redis::cmd("PUBLISH")
        .arg("ficnexus:rt")
        .arg(&payload)
        .query_async::<()>(&mut conn)
        .await?;
    Ok(())
}
