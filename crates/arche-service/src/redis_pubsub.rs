use crate::cache::Cache;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::StreamExt;
use tracing::{info, warn};
use uuid::Uuid;

pub const INVALIDATION_CHANNEL: &str = "arche:cache-invalidate";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct InvalidationMessage {
    #[serde(rename = "type")]
    msg_type: String,
    client_id: String,
}

impl InvalidationMessage {
    fn new(client_id: Uuid) -> Self {
        Self {
            msg_type: "invalidate".into(),
            client_id: client_id.to_string(),
        }
    }

    fn from_json(payload: &str) -> Option<Uuid> {
        let msg: Self = serde_json::from_str(payload).ok()?;
        if msg.msg_type == "invalidate" {
            Uuid::parse_str(&msg.client_id).ok()
        } else {
            None
        }
    }
}

use std::fmt;

impl fmt::Display for InvalidationMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_default()
        )
    }
}

#[derive(Clone)]
pub struct RedisPubSubHandle {
    tx: mpsc::Sender<Uuid>,
}

impl RedisPubSubHandle {
    pub async fn invalidate(&self, client_id: Uuid) {
        let _ = self.tx.send(client_id).await;
    }
}

pub async fn connect_redis(redis_url: &str) -> Result<Client, redis::RedisError> {
    let client = Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("PING")
        .query_async::<_, String>(&mut conn)
        .await?;
    Ok(client)
}

pub fn start_redis_pubsub(
    client: Client,
    pool: Arc<PgPool>,
    cache: Arc<RwLock<Cache>>,
) -> RedisPubSubHandle {
    let (tx, mut rx) = mpsc::channel::<Uuid>(256);

    let pub_client = client.clone();
    tokio::spawn(async move {
        while let Some(client_id) = rx.recv().await {
            let msg = InvalidationMessage::new(client_id);
            let payload = msg.to_string();
            match pub_client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    if let Err(e) = conn
                        .publish::<_, _, ()>(INVALIDATION_CHANNEL, &payload)
                        .await
                    {
                        warn!(error = %e, client_id = %client_id, "redis: publish failed");
                    } else {
                        info!(client_id = %client_id, "redis: published invalidation");
                    }
                }
                Err(e) => {
                    warn!(error = %e, client_id = %client_id, "redis: publish connect failed");
                }
            }
        }
    });

    let sub_pool = pool.clone();
    let sub_cache = cache.clone();
    tokio::spawn(async move {
        loop {
            #[allow(deprecated)]
            let conn_result = client.get_async_connection().await;
            match conn_result {
                Ok(conn) => {
                    let mut pubsub = conn.into_pubsub();
                    if let Err(e) = pubsub.subscribe(INVALIDATION_CHANNEL).await {
                        warn!(error = %e, "redis: subscribe failed, retrying in 1s");
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }

                    info!("redis: subscriber connected, listening on channel {}", INVALIDATION_CHANNEL);

                    let mut stream = pubsub.on_message();
                    loop {
                        match stream.next().await {
                            Some(msg) => {
                                let payload: String = match msg.get_payload() {
                                    Ok(p) => p,
                                    Err(e) => {
                                        warn!(error = %e, "redis: failed to get message payload");
                                        continue;
                                    }
                                };

                                if let Some(client_id) =
                                    InvalidationMessage::from_json(&payload)
                                {
                                    info!(
                                        client_id = %client_id,
                                        "redis: received invalidation, reloading client"
                                    );

                                    match Cache::fetch_client_data(&sub_pool, client_id)
                                        .await
                                    {
                                        Ok(data) => {
                                            let mut cache = sub_cache.write().await;
                                            cache.apply_client_data(client_id, data);
                                        }
                                        Err(e) => {
                                            warn!(
                                                error = %e,
                                                client_id = %client_id,
                                                "redis: failed to reload client data"
                                            );
                                        }
                                    }
                                }
                            }
                            None => {
                                warn!("redis: subscriber stream ended, reconnecting");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(error = %e, "redis: subscriber connect failed, retrying in 1s");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    });

    RedisPubSubHandle { tx }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalidation_message_format() {
        let client_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let msg = InvalidationMessage::new(client_id);
        let json = msg.to_string();

        let expected = serde_json::json!({
            "type": "invalidate",
            "client_id": "550e8400-e29b-41d4-a716-446655440000",
        });

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_invalidation_message_roundtrip() {
        let client_id = Uuid::new_v4();
        let msg = InvalidationMessage::new(client_id);
        let json = msg.to_string();

        let recovered = InvalidationMessage::from_json(&json);
        assert_eq!(recovered, Some(client_id));
    }

    #[test]
    fn test_parse_invalidation_ignores_unknown_type() {
        let json = serde_json::json!({
            "type": "something-else",
            "client_id": "550e8400-e29b-41d4-a716-446655440000",
        })
        .to_string();

        assert_eq!(InvalidationMessage::from_json(&json), None);
    }

    #[test]
    fn test_parse_invalidation_ignores_missing_type() {
        let json = serde_json::json!({
            "client_id": "550e8400-e29b-41d4-a716-446655440000",
        })
        .to_string();

        assert_eq!(InvalidationMessage::from_json(&json), None);
    }

    #[test]
    fn test_parse_invalidation_invalid_json() {
        assert_eq!(InvalidationMessage::from_json("not json"), None);
    }

    #[test]
    fn test_parse_invalidation_invalid_uuid() {
        let json = serde_json::json!({
            "type": "invalidate",
            "client_id": "not-a-uuid",
        })
        .to_string();

        assert_eq!(InvalidationMessage::from_json(&json), None);
    }

    #[test]
    fn test_parse_invalidation_empty_json() {
        assert_eq!(InvalidationMessage::from_json("{}"), None);
    }

    #[test]
    fn test_invalidation_channel_constant() {
        assert_eq!(INVALIDATION_CHANNEL, "arche:cache-invalidate");
    }

    #[tokio::test]
    async fn test_handle_invalidate_sends_on_channel() {
        let (tx, mut rx) = mpsc::channel::<Uuid>(16);
        let handle = RedisPubSubHandle { tx };

        let client_id = Uuid::new_v4();
        handle.invalidate(client_id).await;

        let received = rx.recv().await;
        assert_eq!(received, Some(client_id));
    }

    #[test]
    fn test_handle_is_clone() {
        let (tx, _rx) = mpsc::channel::<Uuid>(16);
        let handle = RedisPubSubHandle { tx };
        let _clone = handle.clone();
    }
}
