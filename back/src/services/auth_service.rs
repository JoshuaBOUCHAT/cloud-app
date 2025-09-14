use actix_session::Session;
use actix_web::{HttpResponse, web};
use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{AppError, AppResult},
    models::user_model::User,
    shared::{EMAIL_RE, PASSWORD_RE},
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
        if !is_valid_email(&self.email) {
            return Err(AppError::Validation("email invalide".into()));
        }
        if !is_valid_password(&self.password) {
            return Err(AppError::Validation("password invalide".into()));
        }
        Ok(())
    }
}

pub async fn login(session: Session, form: web::Json<LoginCredential>) -> AppResult<HttpResponse> {
    if session.contains_key("user_id") {
        return Ok(HttpResponse::Ok().json("already logged in"));
    }
    let credential = &*form;
    let Some(user) = User::get_from_credential(credential).await? else {
        return Ok(HttpResponse::Unauthorized().json("Credentials incorrect"));
    };
    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", user.id).unwrap();
    session
        .insert("verified", user.verified_at.is_some())
        .unwrap();

    return Ok(HttpResponse::Ok().json("login successfull"));
}

fn is_valid_email(email: &str) -> bool {
    EMAIL_RE.is_match(email)
}

fn is_valid_password(password: &str) -> bool {
    PASSWORD_RE.is_match(password)
}
pub fn send_verification_email(user_email: &str, token: &str) -> AppResult<()> {
    let verify_url = format!("https://localhost/verify?token={}", token);

    let html_template = include_str!("../templates/verification_email.html");
    let html = html_template.replace("__VERIFY_URL__", &verify_url);

    send_mail(user_email, "Vérifiez votre adresse email", html)?;

    Ok(())
}

pub async fn register(
    session: Session,
    form: web::Json<LoginCredential>,
) -> AppResult<HttpResponse> {
    if session.contains_key("user_id") {
        return Ok(HttpResponse::Ok().json("already logged in"));
    }
    let credential = &*form;

    credential.is_valide_credential()?;

    let user_id = User::create(credential).await?;

    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", user_id).unwrap();
    session.insert("verified", false).unwrap();

    create_verification_token_and_send_mail(user_id, &credential.email).await?;

    Ok(HttpResponse::Ok().json("Acount creation succed"))
}

async fn create_verification_token_and_send_mail(user_id: i32, email: &str) -> AppResult<()> {
    let token = VerifyToken::new();
    let value = VerifyValue::new(user_id);

    token.send_to_cache(&value).await?;
    send_verification_email(email, &token.token)?;
    Ok(())
}

#[derive(Deserialize)]
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

impl ToRedisArgs for VerifyValue {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        let json = serde_json::to_string(self).expect("User serialization failed");
        out.write_arg(json.as_bytes());
    }
}

impl FromRedisValue for VerifyValue {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        let s: String = redis::from_redis_value(v)?;
        let user: Self = serde_json::from_str(&s).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Failed to deserialize User from JSON",
                e.to_string(),
            ))
        })?;
        Ok(user)
    }
}

pub async fn verify(token: web::Query<VerifyToken>) -> HttpResponse {
    let verify_value: VerifyValue = match redis_get(&token.token).await {
        Err(err) => {
            eprintln!("Error occurs during verify process:\n{err}");
            return HttpResponse::InternalServerError().finish();
        }
        Ok(None) => return HttpResponse::NotFound().json("verification link is wrong or expired "),
        Ok(Some(val)) => val,
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let user_id = verify_value.user_id;
    if now > verify_value.exp {
        //gérer le token expired
        let _ = redis_del(&token.token).await;
        let user = match User::get(user_id).await {
            Ok(Some(user)) => user,
            Ok(None) => return HttpResponse::NotFound().json("User not found"),
            Err(err) => {
                eprintln!("Error happen:{err}\n");
                return HttpResponse::InternalServerError().json("DB error");
            }
        };
        if let Err(err) = create_verification_token_and_send_mail(user_id, &user.email).await {
            eprintln!("Error happen:{err}");
            return HttpResponse::InternalServerError().finish();
        }

        return HttpResponse::Unauthorized().json("token expired");
    }

    if let Err(err) = User::verify_user(user_id).await {
        eprintln!("Error occurs when updating verification for user {user_id}:\n{err}");
        return HttpResponse::InternalServerError().json("error while validating account");
    }
    HttpResponse::Ok().json("account validated")
}
