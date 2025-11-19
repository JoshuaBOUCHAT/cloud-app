use actix_web::{FromRequest, HttpRequest, dev::Payload};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize, Serializer};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SECRET: &[u8; 44] = b"laOOVyHM6s3IcgDAty1O7AXAdRZR6eaaQi65v3qhVRg=";

pub enum TokenError {
    Expired,
    Invalid,
}

#[repr(transparent)]
#[derive(Deserialize, Debug, Clone)]

pub struct Token {
    #[serde(skip_serializing)]
    token_str: String,
}

use serde_json::Value;

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
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub is_user_admin: bool,
    pub exp: u64,
}

impl Claims {
    fn new(user_id: i32, admin: bool) -> Self {
        const HOUR: u64 = 60 * 60;
        let exp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(1 * HOUR);
        Claims {
            user_id,
            exp: exp.as_secs(),
            is_user_admin: admin,
        }
    }
    pub fn new_user_claim(user_id: i32) -> Self {
        Self::new(user_id, false)
    }
    pub fn new_admin_claim(user_id: i32) -> Self {
        Self::new(user_id, true)
    }
    pub fn parse_and_validate(token: &str) -> Result<Claims, TokenError> {
        // CONFIGURATION DE LA VALIDATION
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true; // <-- ENABLE AUTOMATIC VALIDATION

        let valid_claims = decode::<Claims>(token, &DecodingKey::from_secret(SECRET), &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                _ => TokenError::Invalid,
            })?
            .claims;

        Ok(valid_claims)
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

use futures_util::future::{Ready, ready};

use crate::{
    constants::messages::{TOKEN_EXPIRED, TOKEN_INVALID},
    errors::AppError,
};

impl FromRequest for Claims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        Self::extract(req)
    }
    fn extract(req: &HttpRequest) -> Self::Future {
        let maybe_auth_header = try_extract_bearer_header(&req);

        let response = match maybe_auth_header {
            Some(maybe_token_str) => match Claims::parse_and_validate(&maybe_token_str) {
                Ok(claims) => Ok(claims),
                Err(TokenError::Expired) => Err(actix_web::error::ErrorForbidden(TOKEN_EXPIRED)),
                Err(TokenError::Invalid) => Err(actix_web::error::ErrorUnauthorized(TOKEN_INVALID)),
            },
            None => Err(actix_web::error::ErrorUnauthorized(
                "Missing Authorization header",
            )),
        };
        ready(response)
    }
}
fn try_extract_bearer_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| {
            let mut parts = s.splitn(2, ' ');
            match (parts.next(), parts.next()) {
                (Some(scheme), Some(token))
                    if scheme.eq_ignore_ascii_case("Bearer") && !token.trim().is_empty() =>
                {
                    Some(token.trim().to_string())
                }
                _ => None,
            }
        })
}
