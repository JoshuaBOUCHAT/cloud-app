use actix_session::Session;
use actix_web::{HttpResponse, http::StatusCode, web};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{
        auth_models::{VerifyToken, VerifyValue},
        bearer_manager::{Claims, Token},
    },
    constants::messages::{
        CREDENTIALS_INCORECT, TOKEN_EXPIRED, USER_CREATED, USER_LOGGED_OUT, USER_NOT_FOUND,
        USER_NOT_VERIFIED, USER_VERIFIED,
    },
    errors::{AppError, AppResult},
    models::user_model::User,
    shared::{EMAIL_RE, JsonResponse, PASSWORD_RE, get_now_unix},
    utils::{
        email_utils::send_mail,
        redis_utils::{redis_del, redis_get, redis_set},
    },
};

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct LoginCredential {
    email: String,
    password: String,
}
impl LoginCredential {
    pub fn get_email(&self) -> &str {
        &self.email
    }
    pub fn get_password(&self) -> &str {
        &self.password
    }
    fn is_valide_credential(&self) -> AppResult<()> {
        if !Self::is_valid_email(&self.email) {
            return Err(AppError::Validation("email invalid".into()));
        }
        if !Self::is_valid_password(&self.password) {
            return Err(AppError::Validation("password invalid".into()));
        }
        Ok(())
    }
    fn is_valid_email(email: &str) -> bool {
        EMAIL_RE.is_match(email).unwrap()
    }

    fn is_valid_password(password: &str) -> bool {
        PASSWORD_RE.is_match(password).unwrap()
    }
}

pub async fn login(
    session: Session,
    web::Json(credentials): web::Json<LoginCredential>,
) -> AppResult<JsonResponse> {
    // --- Case 1 : User already connected ---
    if let Some(user_id) = session.get::<i32>("user_id").unwrap() {
        return if let Some(user_token) = User::get_token(user_id).await? {
            //Just give back the token
            JsonResponse::ok().token(user_token)
        } else {
            JsonResponse::ok().message(USER_NOT_VERIFIED)
        };
    }
    // --- Case 2 : User not connected, trying to login ---
    let Some(user) = User::get_from_credential(&credentials).await? else {
        //Case 2-1 User connexion failed
        return JsonResponse::unauthorized().message(CREDENTIALS_INCORECT);
    };

    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", user.id).unwrap();

    // --- Step 3 : Distinguish verified / unverified users ---
    let Some(token) = User::get_token(user.id).await? else {
        return JsonResponse::ok().message(USER_NOT_VERIFIED);
    };

    JsonResponse::token(token)
}

pub async fn register(
    session: Session,
    web::Json(credentials): web::Json<LoginCredential>,
) -> AppResult<JsonResponse> {
    if let Some(user_id) = session.get::<i32>("user_id").unwrap() {
        return if let Some(token) = User::get_token(user_id).await? {
            //user have a token so he have been verified
            JsonResponse::token(token)
        } else {
            //user logged in but not verified
            JsonResponse::status(StatusCode::CONFLICT).message("User already connected")
        };
    }

    credentials.is_valide_credential()?;

    let user_id = User::create(&credentials).await?;

    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", user_id).unwrap();

    create_verification_token_and_send_mail(user_id, &credentials.email).await?;

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
pub async fn verify(
    session: Session,
    Query(verify_token): Query<VerifyToken>,
) -> AppResult<JsonResponse> {
    eprintln!("here ! heheh");
    let actual_maybe_id = session.get::<i32>("user_id").unwrap();
    //Check if the current user is not already connected
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
            return JsonResponse::not_found().message(USER_NOT_FOUND);
        };
        create_verification_token_and_send_mail(user_id, &user.email).await?;

        return JsonResponse::unauthorized().message(TOKEN_EXPIRED);
    }

    User::verify_user(user_id).await?;

    let Some(actual_user_id) = actual_maybe_id else {
        //The user verify his account with a not connected device
        return JsonResponse::ok().message(USER_VERIFIED);
    };
    if verify_value.get_user_id() != actual_user_id {
        //A connected user verfiy another account so return success but not a new token
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

pub async fn logout(session: Session) -> HttpResponse {
    let _ = session.remove("user_id");
    HttpResponse::Ok().json(USER_LOGGED_OUT)
}
