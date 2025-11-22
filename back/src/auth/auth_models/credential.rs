use std::sync::LazyLock;

use fancy_regex::Regex;
use serde::Deserialize;

use crate::errors::{AppError, AppResult};

pub static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w{2,}$").unwrap());
pub static PASSWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^A-Za-z\d]).{8,}$").unwrap()
});

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
    pub fn is_valide_credential(&self) -> AppResult<()> {
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
