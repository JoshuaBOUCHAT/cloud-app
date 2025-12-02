use serde::{Deserialize, Serialize};

use crate::{
    auth::auth_models::{
        internal_user_claim::InternalUserClaim,
        token::{ExpiredAbleTokenError, ExpiredTokenAble, TokenAble, TokenError},
    },
    errors::AppResult,
    utils::redis_utils::{redis_del, redis_get, redis_set},
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
    pub async fn send_to_cache(&self, internal_user_claim: &InternalUserClaim) -> AppResult<()> {
        redis_set(&self.key, &internal_user_claim.encode()?).await?;
        Ok(())
    }
    pub async fn get_from_cache(&self) -> AppResult<ResetResult> {
        let Some(raw_token) = redis_get::<String, String>(&self.key).await? else {
            return Ok(ResetResult::Invalide);
        };
        match InternalUserClaim::decode_expired(&raw_token) {
            Ok(val) => Ok(ResetResult::Ok(val.get_user_id())),
            Err(ExpiredAbleTokenError::EncodeError(m)) => Err(TokenError::EncodeError(m))?,
            Err(ExpiredAbleTokenError::Invalid) => Ok(ResetResult::Invalide),
            Err(ExpiredAbleTokenError::ExpiredId(id)) => Ok(ResetResult::Expired(id)),
        }
    }
    pub async fn invalidate(&self) -> AppResult<()> {
        redis_del(&self.key).await
    }
}
impl AsRef<str> for CacheKey {
    fn as_ref(&self) -> &str {
        &self.key
    }
}
