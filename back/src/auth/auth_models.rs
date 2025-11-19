use serde::{Deserialize, Serialize};

use crate::{errors::AppResult, shared::get_now_unix, utils::redis_utils::redis_set};

#[derive(Deserialize, Serialize)]
pub struct VerifyToken {
    token: String,
}
impl VerifyToken {
    pub fn new() -> Self {
        Self {
            token: uuid::Uuid::new_v4().to_string(),
        }
    }
    pub async fn send_to_cache(&self, verify_value: &VerifyValue) -> AppResult<()> {
        redis_set(&self.token, verify_value).await?;
        Ok(())
    }
}
impl AsRef<str> for VerifyToken {
    fn as_ref(&self) -> &str {
        &self.token
    }
}

#[derive(Serialize, Deserialize)]
pub struct VerifyValue {
    user_id: i32,
    exp: u64,
}

impl VerifyValue {
    pub fn new(user_id: i32) -> Self {
        const MINUTE_SECS: u64 = 60;
        let exp = get_now_unix() + 15 * MINUTE_SECS;
        Self { user_id, exp }
    }
    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }
    pub fn get_exp(&self) -> u64 {
        self.exp
    }
}
