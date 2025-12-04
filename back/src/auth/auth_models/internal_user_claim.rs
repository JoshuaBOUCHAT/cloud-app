use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    auth::auth_models::token::{ExpiredTokenAble, TokenAble},
    shared::get_now_unix,
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
