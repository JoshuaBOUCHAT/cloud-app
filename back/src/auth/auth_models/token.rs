use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize, Serializer, de::DeserializeOwned};
use serde_json::Value;

use crate::SECRET;
#[derive(Debug)]
pub enum TokenError {
    Expired,
    Invalid,
    EncodeError(String),
}

#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]

pub struct Token {
    token_str: String,
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.token_str
    }
}

use jsonwebtoken::errors::ErrorKind::ExpiredSignature;
pub trait TokenAble: Serialize + DeserializeOwned {
    fn encode(&self) -> Result<Token, TokenError> {
        let header = Header::new(Algorithm::HS256);
        let token_str = encode(&header, self, &EncodingKey::from_secret(SECRET))
            .map_err(|err| TokenError::EncodeError(err.to_string()))?;
        Ok(Token { token_str })
    }
    fn decode(raw_token: &str) -> Result<Self, TokenError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true; // <-- ENABLE AUTOMATIC VALIDATION

        match decode::<Self>(raw_token, &DecodingKey::from_secret(SECRET), &validation) {
            Ok(data) => Ok(data.claims),
            Err(e) if e.kind() == &ExpiredSignature => Err(TokenError::Expired),
            Err(_) => Err(TokenError::Invalid),
        }
    }
}

#[derive(Debug)]
pub enum ExpiredAbleTokenError {
    ExpiredId(i32),
    Invalid,
    EncodeError(String),
}

pub trait ExpiredTokenAble: TokenAble {
    fn get_user_id(&self) -> i32;

    fn decode_expired(raw_token: &str) -> Result<Self, ExpiredAbleTokenError> {
        match Self::decode(raw_token) {
            Ok(decoded) => Ok(decoded),
            Err(TokenError::Expired) => {
                let mut validation_no_exp = Validation::new(Algorithm::HS256);
                validation_no_exp.validate_exp = false;

                let token_data = decode::<Self>(
                    raw_token,
                    &DecodingKey::from_secret(SECRET),
                    &validation_no_exp,
                )
                .map_err(|_| ExpiredAbleTokenError::Invalid)?;
                Err(ExpiredAbleTokenError::ExpiredId(
                    token_data.claims.get_user_id(),
                ))
            }
            Err(TokenError::Invalid) => Err(ExpiredAbleTokenError::Invalid),
            Err(TokenError::EncodeError(msg)) => Err(ExpiredAbleTokenError::EncodeError(msg)),
        }
    }
}
