use crate::APP_URL;
use crate::auth::auth_models::verify::ResetResult;
use crate::{
    auth::auth_models::{
        auth_state::{self, AuthState},
        credential::{Email, RawEmail, RawLoginCredential},
        refresh_token::{REFRESH_TOKEN_KEY, RefreshToken},
        token::{ExpiredTokenAble, TokenAble, TokenError},
        verify::{VerifyKey, VerifyValue},
    },
    constants::messages::{
        CREDENTIALS_INCORECT, EMAIL_ALREADY_EXIST, USER_LOGGED_OUT, USER_NOT_LOGIN,
        USER_NOT_VERIFIED, USER_VERIFIED,
    },
    errors::{AppError, AppResult},
    models::user_model::User,
    shared::{APIResponse, JsonResponse},
    utils::{
        email_utils::send_mail,
        redis_utils::{redis_del, redis_get},
    },
};
use actix_session::{Session, SessionExt, SessionInsertError};
use actix_web::web::Json;
use actix_web::{HttpRequest, http::StatusCode, post};

#[post("/login")]
pub async fn login(req: HttpRequest, raw_credentials: Json<RawLoginCredential>) -> APIResponse {
    // --- Case 1 : User already connected ---

    match auth_state::try_extract_auth_state(&req).await? {
        AuthState::Connected(claims) => {
            return JsonResponse::ok().token(claims.encode()?);
        }
        AuthState::NotVerified(_) => return JsonResponse::ok().message(USER_NOT_VERIFIED),
        AuthState::Guess => {}
    }

    // --- Case 2 : User not connected, trying to login ---

    let credentials = raw_credentials.into_inner().verify()?;
    let Some(user) = User::get_from_credential(&credentials).await? else {
        return JsonResponse::unauthorized().message(CREDENTIALS_INCORECT);
    };
    req.get_session()
        .insert(REFRESH_TOKEN_KEY, RefreshToken::new(user.id))
        .map_err(map_session_insert_error)?;

    // --- Step 3 : Distinguish verified / unverified users ---
    let Some(token) = User::get_token(user.id).await? else {
        return JsonResponse::ok().message(USER_NOT_VERIFIED);
    };

    JsonResponse::token(token)
}
fn map_session_insert_error(err: SessionInsertError) -> AppError {
    let err_msg =
        format!("An error occurs in auth::auth_service when inserting in session err:\n{err}");
    AppError::Internal(err_msg)
}

#[post("/register")]
pub async fn register(req: HttpRequest, raw_credentials: Json<RawLoginCredential>) -> APIResponse {
    // --- Case 1 : User already connected ---

    match auth_state::try_extract_auth_state(&req).await? {
        AuthState::Connected(claims) => {
            return JsonResponse::ok().token(claims.encode()?);
        }
        AuthState::NotVerified(_) => return JsonResponse::ok().message(USER_NOT_VERIFIED),
        AuthState::Guess => {}
    }
    let credentials = raw_credentials.into_inner().verify()?;

    let user_id = match User::create(&credentials).await {
        Err(AppError::Conflict(msg)) if msg == EMAIL_ALREADY_EXIST => {
            return JsonResponse::status(StatusCode::CONFLICT).message(EMAIL_ALREADY_EXIST);
        }
        others => others?,
    };

    let rfresh_token = RefreshToken::new(user_id);
    req.get_session()
        .insert(REFRESH_TOKEN_KEY, rfresh_token.encode()?)
        .map_err(map_session_insert_error)?;

    create_verification_token_and_send_mail(user_id, &credentials.get_email()).await?;

    JsonResponse::ok().message(USER_NOT_VERIFIED)
}

async fn create_verification_token_and_send_mail(user_id: i32, email: &Email) -> AppResult<()> {
    let key = VerifyKey::new();
    let value = VerifyValue::new(user_id);

    key.send_to_cache(&value).await?;
    send_verification_email(email, &key)
}
pub fn send_verification_email(user_email: &Email, key: &VerifyKey) -> AppResult<()> {
    let verify_url = format!("https://localhost/api/auth/verify?token={}", key.as_ref());

    let html_template = include_str!("../templates/verification_email.html");
    let html = html_template.replace("__VERIFY_URL__", &verify_url);

    send_mail(user_email, "Vérifiez votre adresse email", html)
}

use crate::auth::auth_models::token::ExpiredAbleTokenError;

#[post("/verify")]
pub async fn verify(auth_state: AuthState, Json(verify_key): Json<VerifyKey>) -> APIResponse {
    eprintln!("here ! heheh");
    // --- Case 1 : User already connected ---
    let maybe_session_id = match auth_state {
        AuthState::Connected(claims) => {
            return JsonResponse::ok().token(claims.encode()?);
        }
        AuthState::NotVerified(user_id) => Some(user_id),
        AuthState::Guess => None,
    };

    let Some(raw_verify_value): Option<String> = redis_get(&verify_key).await? else {
        return JsonResponse::not_found().message("link invalid or expired");
    };
    // Token used so no longer usefull
    redis_del(&verify_key).await?;

    let verify_user_id: i32 = match VerifyValue::decode_expired(&raw_verify_value) {
        Ok(val) => val.get_user_id(),
        Err(ExpiredAbleTokenError::ExpiredId(user_id)) => return handle_expired_link(user_id).await,
        Err(ExpiredAbleTokenError::EncodeError(m)) => Err(TokenError::EncodeError(m))?,
        Err(_) => return Err(TokenError::Invalid)?,
    };

    User::verify_user(verify_user_id).await?;

    if maybe_session_id.is_none_or(|session_id| session_id != verify_user_id) {
        return JsonResponse::ok().message(USER_VERIFIED);
    }
    // --- Case where user is connected and verified his account
    let token = User::get_token(verify_user_id)
        .await?
        .ok_or(AppError::Internal(format!(
            "Internal error occurs when trying to retreive User n°{}'s token ",
            verify_user_id
        )))?;
    JsonResponse::ok().token(token)
}
async fn handle_expired_link(user_id: i32) -> APIResponse {
    let Some(user) = User::get(user_id).await? else {
        let err_message = format!("Verification token exists for non-existing user_id={user_id}");
        return Err(AppError::Internal(err_message));
    };
    create_verification_token_and_send_mail(user_id, &user.email).await?;

    return JsonResponse::invalid_token();
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

    let email = raw_email.verify()?;
    if let Some(user_id) = User::get_user_id_from_mail(&email).await? {
        create_reset_token_and_send_mail(user_id, &email).await?;
    };

    JsonResponse::ok().empty()
}
async fn create_reset_token_and_send_mail(user_id: i32, email: &Email) -> AppResult<()> {
    let key = VerifyKey::new();
    let value = VerifyValue::new(user_id);

    key.send_to_cache(&value).await?;
    send_reset_email(email, &key)
}
pub fn send_reset_email(user_email: &Email, key: &VerifyKey) -> AppResult<()> {
    let reset_url = format!("{APP_URL}/auth/reset?token={}", key.as_ref());

    let html_template = include_str!("../templates/reset_password_email.html");
    let html = html_template.replace("__RESET_URL__", &reset_url);

    send_mail(user_email, "Réinitialiser votre mot de passe", html)
}

#[post("/reset/validate")]
pub async fn change_password_validate(Json(key): Json<VerifyKey>) -> APIResponse {
    let user_id = match key.get_from_cache().await? {
        ResetResult::Invalide => return JsonResponse::not_found().empty(),
        ResetResult::Expired(user_id) => return handle_expired_key(user_id).await,
        ResetResult::Ok(user_id) => user_id,
    };
}

async fn ensure_expiration(user_id: i32, key: &VerifyKey) -> APIResponse {
    VerifyValue::
}

async fn handle_expired_key(user_id: i32) -> APIResponse {
    todo!()
}

#[post("/reset")]
pub async fn change_password() -> APIResponse {
    todo!()
}
