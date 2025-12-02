use std::pin::Pin;

use actix_web::{FromRequest, HttpRequest};

use crate::{
    auth::auth_models::{
        claims::{self, Claims, try_extract_claims},
        refresh_token::{REFRESH_TOKEN_KEY, RefreshClaim, handle_session_error},
        token::TokenAble,
    },
    errors::{AppError, AppResult},
    models::user_model::User,
};

pub enum AuthState {
    Connected(Claims),
    NotVerified(i32),
    Guess,
}
impl FromRequest for AuthState {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();
        Box::pin(async move { try_extract_auth_state(&req).await })
    }
    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        let req = req.clone();
        Box::pin(async move { try_extract_auth_state(&req).await })
    }
}
use actix_session::SessionExt;
pub async fn try_extract_auth_state(req: &HttpRequest) -> AppResult<AuthState> {
    if let Ok(claims) = try_extract_claims(req) {
        return Ok(AuthState::Connected(claims));
    }
    let maybe_unchecked_token_str: Option<String> = req
        .get_session()
        .get::<String>(REFRESH_TOKEN_KEY)
        .map_err(handle_session_error)?;
    let Some(unchecked_token_str) = maybe_unchecked_token_str else {
        return Ok(AuthState::Guess);
    };
    let Ok(refresh_token) = RefreshClaim::decode(&unchecked_token_str) else {
        return Ok(AuthState::Guess);
    };
    //User maybe already verified but not having token
    if let Some(claims) = User::get_claim(refresh_token.get_user_id()).await? {
        return Ok(AuthState::Connected(claims));
    }

    Ok(AuthState::NotVerified(refresh_token.get_user_id()))
}
