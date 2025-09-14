use crate::errors::AppResult;
use fancy_regex::Regex;
use std::sync::LazyLock;

#[allow(async_fn_in_trait)]
pub trait SQLable {
    async fn up() -> AppResult<()>;
    async fn down() -> AppResult<()>;
}
pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

pub static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w{2,}$").unwrap());
pub static PASSWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^A-Za-z\d]).{8,}$").unwrap()
});
