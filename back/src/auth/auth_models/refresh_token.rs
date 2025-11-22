use std::str::FromStr;

use actix_web::{FromRequest, HttpRequest};
use futures_util::future::ready;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    SECRET,
    auth::auth_models::token::{Token, TokenError},
    constants::messages::TOKEN_INVALID,
    errors::{AppError, AppResult},
    shared::get_now_unix,
};
pub const REFRESH_TOKEN_KEY: &str = "refresh_token";

#[derive(Serialize, Deserialize)]
pub struct RefreshToken {
    user_id: i32,
    exp: u64,
}
impl RefreshToken {
    pub fn new(user_id: i32) -> Self {
        const DAY_SECS: u64 = 60 * 60 * 24;
        let exp = get_now_unix() + 7 * DAY_SECS;
        Self { user_id, exp }
    }
    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }
}
impl FromStr for RefreshToken {
    type Err = TokenError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true; // <-- ENABLE AUTOMATIC VALIDATION

        Ok(
            jsonwebtoken::decode(s, &DecodingKey::from_secret(SECRET), &validation)
                .map_err(|e| match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                    _ => TokenError::Invalid,
                })?
                .claims,
        )
    }
}

/// error 500 on session error should not happend
/// Forbiden on
impl FromRequest for RefreshToken {
    type Error = AppError;
    type Future = futures_util::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        Self::extract(req)
    }
    fn extract(req: &HttpRequest) -> Self::Future {
        ready(try_extract_refresh_token_from_req(req))
    }
}
use actix_session::{SessionExt, SessionGetError};

pub fn try_extract_refresh_token_from_req(req: &HttpRequest) -> AppResult<RefreshToken> {
    let maybe_unchecked_token_str: Option<String> = req
        .get_session()
        .get::<String>(REFRESH_TOKEN_KEY)
        .map_err(handle_session_error)?;
    let unchecked_token_str = maybe_unchecked_token_str.ok_or(TokenError::Absent)?;

    //Parising also validate the token
    let refresh_token = unchecked_token_str.parse()?;

    Ok(refresh_token)
}
pub fn handle_session_error(err: SessionGetError) -> AppError {
    let err = format!(
        "Error while deserialising a string in auth::auth_models::refresh_token::try_extract_refresh_token_from_req with err:{err}"
    );
    AppError::Internal(err)
}
