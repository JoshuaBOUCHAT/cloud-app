use crate::auth::auth_models::auth_state::try_extract_auth_state;
use crate::auth::auth_models::cache_key::CacheKey;
use crate::auth::auth_service::{
    ChangePasswordResult, LoginResult, RegisterResult, ValidateResult, VerifyResult,
    change_password_service, create_verification_token_and_send_mail, forgot_service,
    login_service, register_service, validate_service, verify_service,
};
use crate::constants::messages::{TOKEN_EXPIRED, TOKEN_INVALID};
use crate::{
    auth::auth_models::{
        auth_state::AuthState,
        credential::{RawEmail, RawLoginCredential},
        refresh_token::{REFRESH_TOKEN_KEY, RefreshClaim},
        token::TokenAble,
    },
    constants::messages::{
        CREDENTIALS_INCORECT, EMAIL_ALREADY_EXIST, USER_LOGGED_OUT, USER_NOT_LOGIN,
        USER_NOT_VERIFIED, USER_VERIFIED,
    },
    errors::{AppError, AppResult},
    shared::{APIResponse, JsonResponse},
};
use actix_session::{Session, SessionExt, SessionInsertError};
use actix_web::web::Json;
use actix_web::{HttpRequest, http::StatusCode, post};
use serde::Deserialize;

#[post("/login")]
pub async fn login(req: HttpRequest, raw_credentials: Json<RawLoginCredential>) -> APIResponse {
    // --- Case 1 : User already connected ---
    let auth_state = try_extract_auth_state(&req).await?;
    let login_result = login_service(auth_state, raw_credentials.into_inner()).await?;

    match login_result {
        LoginResult::CredentialsIncorect => {
            JsonResponse::unauthorized().message(CREDENTIALS_INCORECT)
        }
        LoginResult::Connected(token_claims, maybe_refresh_claim) => {
            if let Some(refresh_claim) = maybe_refresh_claim {
                try_insert_refresh_token_in_session(&req.get_session(), &refresh_claim)?;
            }
            JsonResponse::token(token_claims)
        }
        LoginResult::NotVerified(maybe_refresh_claim) => {
            if let Some(refresh_claim) = maybe_refresh_claim {
                try_insert_refresh_token_in_session(&req.get_session(), &refresh_claim)?;
            }
            JsonResponse::ok().message(USER_NOT_VERIFIED)
        }
    }
}
fn try_insert_refresh_token_in_session(
    session: &Session,
    refresh_claim: &RefreshClaim,
) -> AppResult<()> {
    session
        .insert(REFRESH_TOKEN_KEY, refresh_claim.encode()?)
        .map_err(map_session_insert_error)
}
fn map_session_insert_error(err: SessionInsertError) -> AppError {
    let err_msg =
        format!("An error occurs in auth::auth_service when inserting in session err:\n{err}");
    AppError::Internal(err_msg)
}

#[post("/register")]
pub async fn register(req: HttpRequest, raw_credentials: Json<RawLoginCredential>) -> APIResponse {
    // --- Case 1 : User already connected ---
    let auth_state = try_extract_auth_state(&req).await?;

    match register_service(raw_credentials.into_inner(), auth_state).await? {
        RegisterResult::EmailAlreadyExist => {
            JsonResponse::status(StatusCode::CONFLICT).message(EMAIL_ALREADY_EXIST)
        }
        RegisterResult::NotVerified => JsonResponse::ok().message(USER_NOT_VERIFIED),
        RegisterResult::Token(token) => JsonResponse::ok().token(token),
        RegisterResult::NewUser(refresh_claim, email) => {
            try_insert_refresh_token_in_session(&req.get_session(), &refresh_claim)?;
            create_verification_token_and_send_mail(refresh_claim.get_user_id(), &email).await?;
            JsonResponse::ok().message(USER_NOT_VERIFIED)
        }
    }
}

type VerificationKey = CacheKey;
#[post("/verify")]
pub async fn verify(auth_state: AuthState, Json(verify_key): Json<VerificationKey>) -> APIResponse {
    match verify_service(auth_state, verify_key).await? {
        VerifyResult::Invalid => JsonResponse::not_found().message(TOKEN_INVALID),
        VerifyResult::Token(token) => JsonResponse::token(token),
        VerifyResult::Expired => JsonResponse::unauthorized().message(TOKEN_EXPIRED),
        //in case where an already connected user verified another account
        VerifyResult::Verified => JsonResponse::ok().message(USER_VERIFIED),
    }
}

#[post("/refresh_token")]
pub async fn refresh_token(auth_state: AuthState) -> APIResponse {
    match auth_state {
        AuthState::Connected(claims) => JsonResponse::ok().token(claims.encode()?),
        AuthState::NotVerified(_) => JsonResponse::ok().message(USER_NOT_VERIFIED),
        AuthState::Guess => JsonResponse::unauthorized().message(USER_NOT_LOGIN),
    }
}

#[post("/logout")]
pub async fn logout(session: Session) -> APIResponse {
    let _ = session.remove(REFRESH_TOKEN_KEY);
    JsonResponse::ok().message(USER_LOGGED_OUT)
}

#[post("/forgot")]
pub async fn forgot(auth_state: AuthState, Json(raw_email): Json<RawEmail>) -> APIResponse {
    match auth_state {
        AuthState::Connected(claims) => return JsonResponse::ok().token(claims.encode()?),
        AuthState::NotVerified(_) => return JsonResponse::ok().message(USER_NOT_VERIFIED),
        AuthState::Guess => {}
    };
    forgot_service(raw_email).await?;
    JsonResponse::ok().empty()
}

#[post("/reset/validate")]
pub async fn validate(key: Json<CacheKey>) -> APIResponse {
    match validate_service(key.into_inner()).await? {
        ValidateResult::Validate => JsonResponse::ok().empty(),
        ValidateResult::Expired => JsonResponse::unauthorized().empty(),
        ValidateResult::Invalid => JsonResponse::not_found().empty(),
    }
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    key: CacheKey,
    raw_password: String,
}

#[post("/reset/update")]
pub async fn change_password(Json(req): Json<ChangePasswordRequest>) -> APIResponse {
    match change_password_service(&req.key, &req.raw_password).await? {
        ChangePasswordResult::PasswordChanged => JsonResponse::ok().empty(),
        ChangePasswordResult::KeyInvalid => JsonResponse::unauthorized().message(TOKEN_INVALID),
        ChangePasswordResult::PasswordInvalid => {
            JsonResponse::unauthorized().message(CREDENTIALS_INCORECT)
        }

        ChangePasswordResult::KeyExpired => JsonResponse::unauthorized().message(TOKEN_EXPIRED),
    }
}
