use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::OnceLock;

use crate::errors::AppResult;

// Redis pool global initialisé de manière synchrone
pub static REDIS_POOL: OnceLock<bb8::Pool<RedisConnectionManager>> = OnceLock::new();

pub async fn init_redis_pool() {
    let manager = RedisConnectionManager::new("redis://redis/").unwrap();
    REDIS_POOL
        .set(
            bb8::Pool::builder()
                .build(manager)
                .await
                .expect("can't build the redis pool"),
        )
        .expect("set error in redis pool");
}

/// Set a value in Redis with expiration (seconds)
pub async fn redis_set_ex<K: Serialize, V: Serialize>(
    key: &K,
    value: &V,
    seconds: u64,
) -> AppResult<()> {
    let key_str = serde_json::to_string(key)?;
    let value_str = serde_json::to_string(value)?;
    let mut conn = REDIS_POOL.get().unwrap().get().await?;
    let (): () = conn.set_ex(key_str, value_str, seconds).await?;
    Ok(())
}

/// Set a value in Redis without expiration
pub async fn redis_set<K: Serialize, V: Serialize>(key: &K, value: &V) -> AppResult<()> {
    let key_str = serde_json::to_string(key)?;
    let value_str = serde_json::to_string(value)?;
    let mut conn = REDIS_POOL.get().unwrap().get().await?;
    let (): () = conn.set(key_str, value_str).await?;
    Ok(())
}

/// Get a value from Redis
pub async fn redis_get<K: Serialize, V: DeserializeOwned>(key: &K) -> AppResult<Option<V>> {
    let key_str = serde_json::to_string(key)?;
    let mut conn = REDIS_POOL.get().unwrap().get().await?;
    let value: Option<String> = conn.get(key_str).await?;
    match value {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

/// Delete a key from Redis
pub async fn redis_del<K: Serialize>(key: &K) -> AppResult<()> {
    let key_str = serde_json::to_string(key)?;
    let mut conn = REDIS_POOL.get().unwrap().get().await?;
    let _: () = conn.del(key_str).await?;
    Ok(())
}
