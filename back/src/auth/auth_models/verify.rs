use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    auth::auth_models::token::{ExpiredAbleTokenError, ExpiredTokenAble, TokenAble, TokenError},
    errors::AppResult,
    shared::get_now_unix,
    utils::redis_utils::{redis_get, redis_set},
};

#[derive(Deserialize, Serialize)]
pub struct CacheKey {
    key: String,
}

pub enum ResetResult {
    Ok(i32),
    Expired(i32),
    Invalide,
}

impl CacheKey {
    pub fn new() -> Self {
        Self {
            key: uuid::Uuid::new_v4().to_string(),
        }
    }
    pub async fn send_to_cache(&self, verify_value: &VerifyValue) -> AppResult<()> {
        redis_set(&self.key, &verify_value.encode()?).await?;
        Ok(())
    }
    pub async fn get_from_cache(&self) -> AppResult<ResetResult> {
        let Some(raw_token) = redis_get::<String, String>(&self.key).await? else {
            return Ok(ResetResult::Invalide);
        };
        match VerifyValue::decode_expired(&raw_token) {
            Ok(val) => Ok(ResetResult::Ok(val.get_user_id())),
            Err(ExpiredAbleTokenError::EncodeError(m)) => Err(TokenError::EncodeError(m))?,
            Err(ExpiredAbleTokenError::Invalid) => Ok(ResetResult::Invalide),
            Err(ExpiredAbleTokenError::ExpiredId(id)) => Ok(ResetResult::Expired(id)),
        }
    }
}
impl AsRef<str> for CacheKey {
    fn as_ref(&self) -> &str {
        &self.key
    }
}

#[derive(Serialize, Deserialize)]
pub struct VerifyValue {
    user_id: i32,
    exp: u64,
}
impl TokenAble for VerifyValue {}
impl ExpiredTokenAble for VerifyValue {
    fn get_user_id(&self) -> i32 {
        self.user_id
    }
}

impl VerifyValue {
    pub fn new(user_id: i32, ttl: Duration) -> Self {
        const MINUTE_SECS: u64 = 60;
        let exp = get_now_unix() + ttl.as_secs();
        Self { user_id, exp }
    }
    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }
    pub fn get_exp(&self) -> u64 {
        self.exp
    }
}
