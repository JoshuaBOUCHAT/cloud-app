use std::sync::LazyLock;

use bb8::RunError;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use sqlx::MySql;
use tokio::runtime;

use crate::{errors::AppResult, shared::DynResult};

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

pub async fn redis_set_ex<'a, K, V>(key: K, value: V, seconds: u64) -> AppResult<()>
where
    K: ToRedisArgs + Send + Sync + 'a,
    V: ToRedisArgs + Send + Sync + 'a,
{
    let mut conn = (*REDIS_POOL).get().await?;
    let (): () = conn.set_ex(key, value, seconds).await?;
    Ok(())
}
pub async fn redis_get<'a, K, V>(key: K) -> AppResult<Option<V>>
where
    K: ToRedisArgs + Send + Sync + 'a,
    V: FromRedisValue + Send + Sync + 'a,
{
    let mut conn = (*REDIS_POOL).get().await?;
    // get retourne RedisResult<Option<V>>
    let value: Option<V> = conn.get(key).await?;
    Ok(value)
}

pub async fn redis_set<'a, K, V>(key: K, value: V) -> AppResult<()>
where
    K: ToRedisArgs + Send + Sync + 'a,
    V: ToRedisArgs + Send + Sync + 'a,
{
    let mut conn = (*REDIS_POOL).get().await?;
    let (): () = conn.set(key, value).await?;
    Ok(())
}
pub async fn redis_del<'a, K>(key: K) -> AppResult<()>
where
    K: ToRedisArgs + Send + Sync + 'a,
{
    let mut conn = (*REDIS_POOL).get().await?;
    let _: () = conn.del(key).await?;
    Ok(())
}
