use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::LazyLock;
use tokio::runtime;

use crate::errors::AppResult;

// Redis pool global initialisé de manière synchrone
static REDIS_POOL: LazyLock<bb8::Pool<RedisConnectionManager>> = LazyLock::new(|| {
    // Runtime temporaire juste pour bloquer l'initialisation async
    let manager = RedisConnectionManager::new("redis://redis/").unwrap();
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can't build runtime for creating redis");
    rt.block_on(async { bb8::Pool::builder().build(manager).await.unwrap() })
});

/// Set a value in Redis with expiration (seconds)
pub async fn redis_set_ex<K: Serialize, V: Serialize>(
    key: &K,
    value: &V,
    seconds: u64,
) -> AppResult<()> {
    let key_str = serde_json::to_string(key)?;
    let value_str = serde_json::to_string(value)?;
    let mut conn = (*REDIS_POOL).get().await?;
    let (): () = conn.set_ex(key_str, value_str, seconds).await?;
    Ok(())
}

/// Set a value in Redis without expiration
pub async fn redis_set<K: Serialize, V: Serialize>(key: &K, value: &V) -> AppResult<()> {
    let key_str = serde_json::to_string(key)?;
    let value_str = serde_json::to_string(value)?;
    let mut conn = (*REDIS_POOL).get().await?;
    let (): () = conn.set(key_str, value_str).await?;
    Ok(())
}

/// Get a value from Redis
pub async fn redis_get<K: Serialize, V: DeserializeOwned>(key: &K) -> AppResult<Option<V>> {
    let key_str = serde_json::to_string(key)?;
    let mut conn = (*REDIS_POOL).get().await?;
    let value: Option<String> = conn.get(key_str).await?;
    match value {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

/// Delete a key from Redis
pub async fn redis_del<K: Serialize>(key: &K) -> AppResult<()> {
    let key_str = serde_json::to_string(key)?;
    let mut conn = (*REDIS_POOL).get().await?;
    let _: () = conn.del(key_str).await?;
    Ok(())
}
