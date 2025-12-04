use crate::APP_URL;
use crate::auth::auth_models::cache_key::ResetResult;
use crate::auth::auth_models::credential::RawEmail;
use crate::shared::is_valid_password;
use crate::{
    auth::auth_models::{
        auth_state::AuthState,
        cache_key::CacheKey,
        credential::{Email, RawLoginCredential},
        internal_user_claim::InternalUserClaim,
        refresh_token::RefreshClaim,
        token::{ExpiredAbleTokenError, ExpiredTokenAble, Token, TokenAble, TokenError},
    },
    constants::messages::EMAIL_ALREADY_EXIST,
    errors::{AppError, AppResult},
    models::user_model::User,
    utils::{
        email_utils::send_mail,
        redis_utils::{redis_del, redis_get},
    },
};
use std::time::Duration;
const EMAIL_LINK_VALDITY_DURATION: Duration = Duration::from_secs(30 * 60);

pub enum RegisterResult {
    Token(Token),
    EmailAlreadyExist,
    NotVerified,
    NewUser(RefreshClaim, Email),
}

pub async fn register_service(
    raw_credentials: RawLoginCredential,
    auth_state: AuthState,
) -> AppResult<RegisterResult> {
    match auth_state {
        AuthState::Connected(claims) => {
            return Ok(RegisterResult::Token(claims.encode()?));
        }
        AuthState::NotVerified(_) => return Ok(RegisterResult::NotVerified),
        AuthState::Guess => {}
    }
    let credentials = raw_credentials.verify()?;

    let user_id = match User::create(&credentials).await {
        Err(AppError::Conflict(msg)) if msg == EMAIL_ALREADY_EXIST => {
            return Ok(RegisterResult::EmailAlreadyExist);
        }
        others => others?,
    };

    let rfresh_token = RefreshClaim::new(user_id);
    return Ok(RegisterResult::NewUser(
        rfresh_token,
        credentials.into_email(),
    ));
}
pub async fn resend_verification_mail(user_id: i32) -> AppResult<()> {
    let email = User::get(user_id).await?.email;
    create_verification_token_and_send_mail(user_id, &email).await
}

pub async fn create_verification_token_and_send_mail(user_id: i32, email: &Email) -> AppResult<()> {
    let key = CacheKey::new();
    println!("key:{}", key.as_ref());
    let value = InternalUserClaim::new(user_id, EMAIL_LINK_VALDITY_DURATION);

    key.send_to_cache(&value).await?;

    dbg!(key.get_from_cache().await.unwrap());
    send_verification_email_with_key(email, &key)
}
fn send_verification_email_with_key(user_email: &Email, key: &CacheKey) -> AppResult<()> {
    let verify_url = format!("{}/auth/verify?token={}", APP_URL, key.as_ref());

    let html_template = include_str!("../templates/verification_email.html");
    let html = html_template.replace("__VERIFY_URL__", &verify_url);

    send_mail(user_email, "Vérifiez votre adresse email", html)
}
pub enum VerifyResult {
    Token(Token),
    Invalid,
    Expired,
    Verified,
}

pub async fn verify_service(
    auth_state: AuthState,
    verify_key: CacheKey,
) -> AppResult<VerifyResult> {
    // --- Case 1 : User already connected ---

    let verified_user_id = match verify_key.get_from_cache().await? {
        ResetResult::Invalide => return Ok(VerifyResult::Invalid),
        ResetResult::Expired(user_id) => {
            verify_key.invalidate().await?;
            handle_expired_verify_link(user_id).await?;
            return Ok(VerifyResult::Expired);
        }
        ResetResult::Ok(user_id) => {
            verify_key.invalidate().await?;
            User::verify_user(user_id).await?;
            user_id
        }
    };

    let user_id = match auth_state {
        AuthState::NotVerified(user_id) if user_id == verified_user_id => user_id,
        _ => return Ok(VerifyResult::Verified),
    };

    // --- Case where user is connected and verified his account
    let token = User::get_token(user_id)
        .await?
        .ok_or(AppError::Internal(format!(
            "Internal error occurs when trying to retreive User n°{}'s token ",
            user_id
        )))?;
    Ok(VerifyResult::Token(token))
}
async fn handle_expired_verify_link(user_id: i32) -> AppResult<()> {
    let Some(user) = User::try_get(user_id).await? else {
        let err_message = format!("Verification token exists for non-existing user_id={user_id}");
        return Err(AppError::Internal(err_message));
    };
    create_verification_token_and_send_mail(user_id, &user.email).await
}

enum DecodeResult {
    Valid(i32),
    Expired(i32),
}

async fn decode_internal(raw_token: &str) -> AppResult<DecodeResult> {
    match InternalUserClaim::decode_expired(&raw_token) {
        Ok(val) => Ok(DecodeResult::Valid(val.get_user_id())),
        Err(ExpiredAbleTokenError::ExpiredId(user_id)) => {
            return Ok(DecodeResult::Expired(user_id));
        }
        Err(ExpiredAbleTokenError::EncodeError(m)) => Err(TokenError::EncodeError(m))?,
        Err(_) => Err(TokenError::Invalid)?,
    }
}

pub enum LoginResult {
    Connected(Token, Option<RefreshClaim>),
    NotVerified(Option<RefreshClaim>),
    CredentialsIncorect,
}

pub async fn login_service(
    auth_state: AuthState,
    raw_credentials: RawLoginCredential,
) -> AppResult<LoginResult> {
    // --- Case 1 : User already connected ---

    match auth_state {
        AuthState::Connected(claims) => {
            return Ok(LoginResult::Connected(claims.encode()?, None));
        }
        AuthState::NotVerified(_) => return Ok(LoginResult::NotVerified(None)),
        AuthState::Guess => {}
    }

    // --- Case 2 : User not connected, trying to login ---

    let credentials = raw_credentials.verify()?;
    let Some(user) = User::get_from_credential(&credentials).await? else {
        return Ok(LoginResult::CredentialsIncorect);
    };

    let refresh_claims = RefreshClaim::new(user.id);

    // --- Step 3 : Distinguish verified / unverified users ---
    if let Some(claims) = User::get_claim(user.id).await? {
        Ok(LoginResult::Connected(
            claims.encode()?,
            Some(refresh_claims),
        ))
    } else {
        Ok(LoginResult::NotVerified(Some(refresh_claims)))
    }
}
pub async fn forgot_service(raw_email: RawEmail) -> AppResult<()> {
    let email = raw_email.verify()?;
    if let Some(user_id) = User::get_user_id_from_email(&email).await? {
        create_reset_token_and_send_email(user_id, &email).await?;
    };

    Ok(())
}

pub enum ValidateResult {
    Expired,
    Validate,
    Invalid,
}

pub async fn validate_service(key: CacheKey) -> AppResult<ValidateResult> {
    let response = match key.get_from_cache().await? {
        ResetResult::Invalide => ValidateResult::Invalid,
        ResetResult::Expired(user_id) => {
            regenerate_reset_token(user_id).await?;
            key.invalidate().await?;
            ValidateResult::Expired
        }
        ResetResult::Ok(user_id) => {
            ensure_expiration(user_id, &key).await?;
            ValidateResult::Validate
        }
    };

    Ok(response)
}
const RESET_TOKEN_VALIDITY_DURATION: Duration = Duration::from_secs(15 * 60);

async fn ensure_expiration(user_id: i32, key: &CacheKey) -> AppResult<()> {
    let reset_claim = InternalUserClaim::new(user_id, RESET_TOKEN_VALIDITY_DURATION);
    key.send_to_cache(&reset_claim).await
}
async fn regenerate_reset_token(user_id: i32) -> AppResult<()> {
    let maybe_user = User::try_get(user_id).await?;
    if let Some(user) = maybe_user {
        create_reset_token_and_send_email(user_id, &user.email).await
    } else {
        Err(AppError::Internal(String::from(
            "Uid in reset cache but the user do not exist",
        )))
    }
}

async fn create_reset_token_and_send_email(user_id: i32, email: &Email) -> AppResult<()> {
    let key = CacheKey::new();
    let reset_claim = get_reset_email_claim(user_id);

    key.send_to_cache(&reset_claim).await?;
    send_reset_email(email, &key)
}
fn get_reset_email_claim(user_id: i32) -> InternalUserClaim {
    InternalUserClaim::new(user_id, EMAIL_LINK_VALDITY_DURATION)
}

fn send_reset_email(user_email: &Email, key: &CacheKey) -> AppResult<()> {
    let reset_url = format!("{APP_URL}/auth/reset?token={}", key.as_ref());

    let html_template = include_str!("../templates/reset_password_email.html");
    let html = html_template.replace("__RESET_URL__", &reset_url);

    send_mail(user_email, "Réinitialiser votre mot de passe", html)
}

pub enum ChangePasswordResult {
    KeyInvalid,
    PasswordChanged,
    KeyExpired,
    PasswordInvalid,
}

pub async fn change_password_service(
    key: &CacheKey,
    raw_password: &str,
) -> AppResult<ChangePasswordResult> {
    let password = if !is_valid_password(raw_password) {
        return Ok(ChangePasswordResult::PasswordInvalid);
    } else {
        raw_password
    };

    let response = match key.get_from_cache().await? {
        ResetResult::Invalide => ChangePasswordResult::KeyInvalid,
        ResetResult::Expired(user_id) => {
            regenerate_reset_token(user_id).await?;
            key.invalidate().await?;
            ChangePasswordResult::KeyExpired
        }
        ResetResult::Ok(user_id) => {
            key.invalidate().await?;
            User::change_password(user_id, password).await?;
            ChangePasswordResult::PasswordChanged
        }
    };
    Ok(response)
}
