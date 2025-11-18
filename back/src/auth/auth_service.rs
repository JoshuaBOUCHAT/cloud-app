use actix_session::Session;
use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};

use crate::{
    auth::bearer_manager::{Claims, Token},
    constants::messages::{
        CREDENTIALS_INCORECT, TOKEN_EXPIRED, USER_CREATED, USER_LOGGED_OUT, USER_NOT_FOUND,
        USER_NOT_VERIFIED, USER_VERIFIED,
    },
    errors::{AppError, AppResult},
    models::user_model::User,
    shared::{EMAIL_RE, JsonResponse, PASSWORD_RE},
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
            return Err(AppError::Validation("email invalide".into()));
        }
        if !Self::is_valid_password(&self.password) {
            return Err(AppError::Validation("password invalide".into()));
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

pub async fn login(session: Session, form: web::Json<LoginCredential>) -> AppResult<HttpResponse> {
    if let Some(user_id) = session.get::<i32>("user_id").unwrap() {
        if let Some(valid_user_token) = User::get_token(user_id).await? {
            return Ok(HttpResponse::Ok().json(JsonResponse::from(valid_user_token)));
        }
    }
    let credential = &*form;
    let Some(user) = User::get_from_credential(credential).await? else {
        return Ok(HttpResponse::Unauthorized()
            .json(JsonResponse::Message(CREDENTIALS_INCORECT.to_string())));
    };
    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", user.id).unwrap();
    return Ok(HttpResponse::Ok().json(JsonResponse::Message(USER_CREATED.to_string())));
}

pub async fn register(
    session: Session,
    form: web::Json<LoginCredential>,
) -> AppResult<HttpResponse> {
    if let Some(user_id) = session.get::<i32>("user_id").unwrap() {
        let response = if let Some(token) = User::get_token(user_id).await? {
            //user have a token so he have been verified
            JsonResponse::from(token)
        } else {
            //user logged in but not verified
            JsonResponse::Message(USER_NOT_VERIFIED.to_string())
        };
        return Ok(HttpResponse::Ok().json(response));
    }
    let credential = &*form;

    credential.is_valide_credential()?;

    let user_id = User::create(credential).await?;

    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", user_id).unwrap();

    create_verification_token_and_send_mail(user_id, &credential.email).await?;

    Ok(HttpResponse::Ok().json(USER_CREATED))
}

async fn create_verification_token_and_send_mail(user_id: i32, email: &str) -> AppResult<()> {
    let token = VerifyToken::new();
    let value = VerifyValue::new(user_id);

    token.send_to_cache(&value).await?;
    send_verification_email(email, &token.token)?;
    Ok(())
}
pub fn send_verification_email(user_email: &str, token: &str) -> AppResult<()> {
    let verify_url = format!("https://localhost/api/auth/verify?token={}", token);

    let html_template = include_str!("../templates/verification_email.html");
    let html = html_template.replace("__VERIFY_URL__", &verify_url);

    send_mail(user_email, "Vérifiez votre adresse email", html)?;

    Ok(())
}

#[derive(Deserialize, Serialize)]
pub struct VerifyToken {
    token: String,
}
impl VerifyToken {
    fn new() -> Self {
        Self {
            token: uuid::Uuid::new_v4().to_string(),
        }
    }
    async fn send_to_cache(&self, verify_value: &VerifyValue) -> AppResult<()> {
        redis_set(&self.token, verify_value).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct VerifyValue {
    user_id: i32,
    exp: u64,
}

impl VerifyValue {
    fn new(user_id: i32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let exp = now + 60 * 15;
        Self { user_id, exp }
    }
}
use web::Query;
pub async fn verify(
    session: Session,
    Query(verify_token): Query<VerifyToken>,
) -> AppResult<HttpResponse> {
    eprintln!("here ! heheh");
    if let Some(user_id) = session.get::<i32>("user_id").unwrap() {
        if let Some(token) = User::get_token(user_id).await? {
            return Ok(HttpResponse::Ok().json(JsonResponse::from(token)));
        }
    }

    let Some(verify_value): Option<VerifyValue> = redis_get(&verify_token).await? else {
        return Ok(
            HttpResponse::NotFound().json(JsonResponse::from_message("link invalid or expired"))
        );
    };
    eprintln!("Token is valide");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let user_id = verify_value.user_id;
    if now > verify_value.exp {
        //gérer le token expired
        redis_del(&verify_token).await?;
        let Some(user) = User::get(user_id).await? else {
            return Ok(HttpResponse::NotFound().json(JsonResponse::from_message(USER_NOT_FOUND)));
        };
        create_verification_token_and_send_mail(user_id, &user.email).await?;

        return Ok(HttpResponse::Unauthorized().json(JsonResponse::from_message(TOKEN_EXPIRED)));
    }

    User::verify_user(user_id).await?;
    Ok(HttpResponse::Ok().json(JsonResponse::from_message(USER_VERIFIED)))
}
pub async fn logout(session: Session) -> HttpResponse {
    let _ = session.remove("user_id");
    HttpResponse::Ok().json(USER_LOGGED_OUT)
}
