use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    auth::auth_models::token::{ExpiredAbleTokenError, ExpiredTokenAble, TokenAble, TokenError},
    errors::AppResult,
    shared::get_now_unix,
    utils::redis_utils::{redis_get, redis_set},
};

#[derive(Serialize, Deserialize)]
pub struct InternalUserClaim {
    user_id: i32,
    exp: u64,
}
impl TokenAble for InternalUserClaim {}
impl ExpiredTokenAble for InternalUserClaim {
    fn get_user_id(&self) -> i32 {
        self.user_id
    }
}

impl InternalUserClaim {
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
