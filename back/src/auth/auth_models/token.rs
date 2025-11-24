use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize, Serializer, de::DeserializeOwned};
use serde_json::Value;

use crate::SECRET;

pub enum TokenError {
    Expired,
    ExpiredId(i32),
    Invalid,
    Absent,
    EncodeError(String),
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

pub trait ExpiredTokenAble: TokenAble {
    fn get_user_id(&self) -> i32;

    fn decode_expired(raw_token: &str) -> Result<Self, TokenError> {
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
                .map_err(|_| TokenError::Invalid)?;
                Err(TokenError::ExpiredId(token_data.claims.get_user_id()))
            }
            Err(err) => Err(err),
        }
    }
}
