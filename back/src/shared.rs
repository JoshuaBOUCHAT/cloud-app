use fancy_regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

use crate::auth::bearer_manager::Token;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

pub static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w{2,}$").unwrap());
pub static PASSWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^A-Za-z\d]).{8,}$").unwrap()
});

#[derive(Serialize)]
pub enum JsonResponse {
    Token(Token),
    Message(String),
}
impl From<Token> for JsonResponse {
    fn from(token: Token) -> Self {
        JsonResponse::Token(token)
    }
}
impl JsonResponse {
    pub fn from_message(msg: impl AsRef<str>) -> Self {
        Self::Message(msg.as_ref().to_string())
    }
}
