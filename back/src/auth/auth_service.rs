use crate::{
    auth::{
        auth_extractor::FromClaim,
        auth_models::{
            auth_state::AuthState,
            credential::LoginCredential,
            refresh_token::{REFRESH_TOKEN_KEY, RefreshToken, handle_session_error},
            token::Token,
            verify::{VerifyToken, VerifyValue},
        },
    },
    constants::messages::{
        CREDENTIALS_INCORECT, EMAIL_ALREADY_EXIST, TOKEN_ABSENT, TOKEN_EXPIRED, USER_LOGGED_OUT,
        USER_NOT_LOGIN, USER_NOT_VERIFIED, USER_VERIFIED,
    },
    errors::{AppError, AppResult},
    models::user_model::User,
    shared::{APIResponse, JsonResponse, get_now_unix},
    utils::{
        email_utils::send_mail,
        redis_utils::{redis_del, redis_get},
    },
};
use actix_session::{Session, SessionInsertError};
use actix_web::{http::StatusCode, post, web};
use serde::Deserialize;
use web::Json;

#[post("/login")]
pub async fn login(
    auth_state: AuthState,
    session: Session,
    Json(credentials): Json<LoginCredential>,
) -> APIResponse {
    // --- Case 1 : User already connected ---

    match auth_state {
        AuthState::Connected(claims) => return JsonResponse::ok().token(Token::try_from(&claims)?),
        AuthState::NotVerified(_) => return JsonResponse::ok().message(USER_NOT_VERIFIED),
        AuthState::Guess => {}
    }

    // --- Case 2 : User not connected, trying to login ---
    let Some(user) = User::get_from_credential(&credentials).await? else {
        return JsonResponse::unauthorized().message(CREDENTIALS_INCORECT);
    };

    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session
        .insert(REFRESH_TOKEN_KEY, RefreshToken::new(user.id))
        .map_err(|err| AppError::Internal(format!("Error")))?;

    // --- Step 3 : Distinguish verified / unverified users ---
    let Some(token) = User::get_token(user.id).await? else {
        return JsonResponse::ok().message(USER_NOT_VERIFIED);
    };

    JsonResponse::token(token)
}
fn map_session_inser_error(err: SessionInsertError) -> AppError {}

#[post("/register")]
pub async fn register(
    maybe_refresh_token: Result<RefreshToken, AppError>,
    session: Session,
    Json(credentials): Json<LoginCredential>,
) -> APIResponse {
    // --- Case 1 : User already connected ---

    match maybe_refresh_token {
        Ok(token) => {
            return if let Some(token) = User::get_token(token.get_user_id()).await? {
                //user have a token so he have been verified
                JsonResponse::token(token)
            } else {
                //user logged in but not verified
                JsonResponse::status(StatusCode::CONFLICT).message("User already connected")
            };
        }
        Err(AppError::Unauthorized(msg)) if msg == TOKEN_ABSENT => {}
        Err(others) => return Err(others),
    }

    credentials.is_valide_credential()?;

    let user_id = match User::create(&credentials).await {
        Err(AppError::Conflict(msg)) if msg == EMAIL_ALREADY_EXIST => {
            return JsonResponse::status(StatusCode::CONFLICT).message(EMAIL_ALREADY_EXIST);
        }
        others => others?,
    };

    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", user_id).unwrap();

    create_verification_token_and_send_mail(user_id, &credentials.get_email()).await?;

    JsonResponse::ok().message(USER_NOT_VERIFIED)
}

async fn create_verification_token_and_send_mail(user_id: i32, email: &str) -> AppResult<()> {
    let token = VerifyToken::new();
    let value = VerifyValue::new(user_id);

    token.send_to_cache(&value).await?;
    send_verification_email(email, token.as_ref())?;
    Ok(())
}
pub fn send_verification_email(user_email: &str, token: &str) -> AppResult<()> {
    let verify_url = format!("https://localhost/api/auth/verify?token={}", token);

    let html_template = include_str!("../templates/verification_email.html");
    let html = html_template.replace("__VERIFY_URL__", &verify_url);

    send_mail(user_email, "Vérifiez votre adresse email", html)?;

    Ok(())
}
use web::Query;

#[post("/verify")]
pub async fn verify(session: Session, Query(verify_token): Query<VerifyToken>) -> APIResponse {
    eprintln!("here ! heheh");
    // --- Case 1 : User already connected ---
    let actual_maybe_id = session.get::<i32>("user_id").unwrap();
    if let Some(user_id) = actual_maybe_id {
        if let Some(token) = User::get_token(user_id).await? {
            return JsonResponse::ok().token(token);
        }
    }
    let Some(verify_value): Option<VerifyValue> = redis_get(&verify_token).await? else {
        return JsonResponse::not_found().message("link invalid or expired");
    };

    eprintln!("Token is valide");
    let user_id = verify_value.get_user_id();

    if get_now_unix() > verify_value.get_exp() {
        //gérer le token expired
        redis_del(&verify_token).await?;
        let Some(user) = User::get(user_id).await? else {
            let err_message =
                format!("Verification token exists for non-existing user_id={user_id}");
            return Err(AppError::Internal(err_message));
        };
        create_verification_token_and_send_mail(user_id, &user.email).await?;

        return JsonResponse::unauthorized().message(TOKEN_EXPIRED);
    }

    User::verify_user(user_id).await?;

    if actual_maybe_id.is_none_or(|actual_user_id| actual_user_id != verify_value.get_user_id()) {
        return JsonResponse::ok().message(USER_VERIFIED);
    }

    let token = User::get_token(user_id)
        .await?
        .ok_or(AppError::Internal(format!(
            "Internal error occurs when trying to retreive User n°{}'s token ",
            user_id
        )))?;
    JsonResponse::ok().token(token)
}

#[post("/refresh_token")]
pub async fn refresh_token(session: Session) -> APIResponse {
    let Some(user_id) = session.get::<i32>("user_id").unwrap_or(None) else {
        return JsonResponse::unauthorized().message(USER_NOT_LOGIN);
    };
    if let Some(token) = User::get_token(user_id).await? {
        JsonResponse::token(token)
    } else {
        JsonResponse::unauthorized().message(USER_NOT_VERIFIED)
    }
}

#[post("/logout")]
pub async fn logout(session: Session) -> APIResponse {
    let _ = session.remove("user_id");
    JsonResponse::ok().message(USER_LOGGED_OUT)
}
