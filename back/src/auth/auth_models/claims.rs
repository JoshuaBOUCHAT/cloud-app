use actix_web::{FromRequest, HttpRequest, dev::Payload};
use futures_util::future::{Ready, ready};
use serde::{Deserialize, Serialize};

use crate::{
    auth::auth_models::token::TokenAble,
    constants::messages::TOKEN_ABSENT,
    errors::{AppError, AppResult},
    shared::get_now_unix,
};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub is_user_admin: bool,
    pub exp: u64,
}

impl TokenAble for Claims {}

impl Claims {
    fn new(user_id: i32, admin: bool) -> Self {
        const HOUR: u64 = 60 * 60;
        let exp = get_now_unix() + 1 * HOUR;
        Claims {
            user_id,
            exp,
            is_user_admin: admin,
        }
    }
    pub fn new_user_claim(user_id: i32) -> Self {
        Self::new(user_id, false)
    }
    pub fn new_admin_claim(user_id: i32) -> Self {
        Self::new(user_id, true)
    }
}

impl FromRequest for Claims {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        Self::extract(req)
    }
    fn extract(req: &HttpRequest) -> Self::Future {
        ready(try_extract_claims(req))
    }
}
pub fn try_extract_claims(req: &HttpRequest) -> AppResult<Claims> {
    let auth_header =
        try_extract_bearer_header(&req).ok_or(AppError::Unauthorized(TOKEN_ABSENT.to_owned()))?;
    Ok(Claims::decode(&auth_header)?)
}

fn try_extract_bearer_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| {
            let mut parts = s.splitn(2, ' ');
            match (parts.next(), parts.next()) {
                (Some(scheme), Some(token))
                    if scheme.eq_ignore_ascii_case("Bearer") && !token.trim().is_empty() =>
                {
                    Some(token.trim().to_string())
                }
                _ => None,
            }
        })
}
