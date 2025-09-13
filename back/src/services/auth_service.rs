use actix_session::Session;
use actix_web::{HttpResponse, web};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::models::user_model::User;

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
}

pub async fn login(session: Session, form: web::Json<LoginCredential>) -> HttpResponse {
    if session.contains_key("user_id") {
        return HttpResponse::Ok().json("already logged in");
    }
    let credential = &*form;
    let user = match User::get_from_credential(credential).await {
        Err(_) => return HttpResponse::InternalServerError().finish(),
        Ok(None) => return HttpResponse::Unauthorized().json("Credentials incorrect"),
        Ok(Some(user)) => user,
    };
    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", user.id).unwrap();
    return HttpResponse::Ok().json("login successfull");
}

use std::sync::LazyLock;

static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w{2,}$").unwrap());
static PASSWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^A-Za-z\d]).{8,}$").unwrap()
});

fn is_valid_email(email: &str) -> bool {
    EMAIL_RE.is_match(email)
}

fn is_valid_password(password: &str) -> bool {
    PASSWORD_RE.is_match(password)
}

pub async fn register(session: Session, form: web::Json<LoginCredential>) -> HttpResponse {
    if session.contains_key("user_id") {
        return HttpResponse::Ok().json("already logged in");
    }
    let credential = &*form;

    if !is_valid_email(&credential.email) {
        return HttpResponse::BadRequest().json("email invalide");
    }
    if !is_valid_password(&credential.password) {
        return HttpResponse::BadRequest().json("password invalide");
    }

    let Ok(maybe_new_user) = User::create(credential).await else {
        return HttpResponse::InternalServerError().finish();
    };
    let id = match maybe_new_user {
        Err(err) => return HttpResponse::Conflict().json(err),
        Ok(id) => id,
    };
    //As the insert fails only if the number can be JSON Serialize we can safely unwrap
    session.insert("user_id", id).unwrap();
    HttpResponse::Ok().json("Acount creation succed")
}
