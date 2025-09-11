use actix_web::{FromRequest, HttpRequest, dev::Payload};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SECRET: &[u8; 44] = b"laOOVyHM6s3IcgDAty1O7AXAdRZR6eaaQi65v3qhVRg=";

pub enum TokenError {
    Expired,
    ParsingError(jsonwebtoken::errors::Error),
}
impl From<jsonwebtoken::errors::Error> for TokenError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::ParsingError(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    token_str: String,
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.token_str
    }
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: i32,
    pub exp: u64,
    pub admin: bool,
}

impl Claims {
    pub fn new(id: i32, admin: bool) -> Self {
        let exp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(60 * 60);
        Claims {
            id,
            exp: exp.as_secs(),
            admin,
        }
    }
    pub fn parse_and_validate(s: &str) -> Result<Self, TokenError> {
        let validation = Validation::new(Algorithm::HS256);
        let decoded_token = decode::<Claims>(s, &DecodingKey::from_secret(SECRET), &validation)?;
        let maybe_expired_claim = decoded_token.claims;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if maybe_expired_claim.exp < now {
            return Err(TokenError::Expired);
        }
        Ok(maybe_expired_claim)
    }
}
impl TryFrom<&Claims> for Token {
    type Error = jsonwebtoken::errors::Error;
    fn try_from(value: &Claims) -> Result<Self, Self::Error> {
        let header = Header::new(Algorithm::HS256);
        let token_str = encode(&header, value, &EncodingKey::from_secret(SECRET))?;
        Ok(Token { token_str })
    }
}

use futures_util::future::{Ready, ready};

impl FromRequest for Claims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .map(|s| s.trim_start_matches("Bearer ").to_string());

        let response = match auth_header {
            Some(maybe_token_str) => match Claims::parse_and_validate(&maybe_token_str) {
                Ok(claims) => Ok(claims),
                Err(TokenError::Expired) => Err(actix_web::error::ErrorForbidden("Token expired")),
                Err(TokenError::ParsingError(_)) => {
                    Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                }
            },
            None => Err(actix_web::error::ErrorUnauthorized(
                "Missing Authorization header",
            )),
        };
        ready(response)
    }
}
