use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::{
    SECRET,
    auth::auth_models::{claims::Claims, refresh_token::RefreshToken},
    errors::AppError,
};

pub enum TokenError {
    Expired,
    Invalid,
    Absent,
}

#[repr(transparent)]
#[derive(Deserialize, Debug, Clone)]

pub struct Token {
    token_str: String,
}

impl Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // token_str contient déjà du JSON, on le parse
        let raw: Value =
            serde_json::from_str(&self.token_str).map_err(serde::ser::Error::custom)?;

        // on renvoie le JSON tel quel
        raw.serialize(serializer)
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.token_str
    }
}
impl TryFrom<&Claims> for Token {
    type Error = AppError;
    fn try_from(value: &Claims) -> Result<Self, Self::Error> {
        let header = Header::new(Algorithm::HS256);
        let token_str = encode(&header, value, &EncodingKey::from_secret(SECRET))?;
        Ok(Token { token_str })
    }
}
impl TryFrom<&RefreshToken> for Token {
    type Error = AppError;
    fn try_from(value: &RefreshToken) -> Result<Self, Self::Error> {
        let header = Header::new(Algorithm::HS256);
        let token_str = encode(&header, value, &EncodingKey::from_secret(SECRET))?;
        Ok(Token { token_str })
    }
}
