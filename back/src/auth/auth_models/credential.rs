use serde::{Deserialize, Serialize};

use crate::{
    constants::messages::CREDENTIALS_INCORECT,
    errors::{AppError, AppResult},
    shared::{is_valid_email, is_valid_password},
};

#[derive(Deserialize)]

pub struct RawLoginCredential {
    email: RawEmail,
    password: String,
}
impl RawLoginCredential {
    pub fn verify(self) -> AppResult<LoginCredential> {
        if is_valid_password(&self.password) {
            Ok(LoginCredential {
                email: self.email.verify()?,
                password: self.password,
            })
        } else {
            Err(AppError::Validation(String::from(CREDENTIALS_INCORECT)))
        }
    }
}

pub struct LoginCredential {
    email: Email,
    password: String,
}
impl LoginCredential {
    pub fn get_email(&self) -> &Email {
        &self.email
    }
    pub fn get_password(&self) -> &str {
        &self.password
    }
    pub fn into_email(self) -> Email {
        self.email
    }
}

#[derive(Deserialize)]
pub struct RawEmail {
    raw_mail: String,
}
impl RawEmail {
    pub fn verify(self) -> AppResult<Email> {
        if is_valid_email(&self.raw_mail) {
            return Ok(Email {
                email: self.raw_mail,
            });
        }
        Err(AppError::Validation(String::from(CREDENTIALS_INCORECT)))
    }
}

use sqlx::{Decode, Encode, MySql, Type};

#[derive(Serialize, Deserialize, Clone, Type)]
#[serde(transparent)]
pub struct Email {
    pub email: String,
}
impl Email {
    pub fn new(email: impl Into<String>) -> AppResult<Self> {
        let email = email.into();
        if is_valid_email(&email) {
            Ok(Self { email })
        } else {
            Err(AppError::Validation(String::from(CREDENTIALS_INCORECT)))
        }
    }
}
impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.email
    }
}
///this trait is should only be use for database deserialisation
impl From<String> for Email {
    fn from(email: String) -> Self {
        Email { email }
    }
}
impl Type<MySql> for Email {
    fn type_info() -> <MySql as sqlx::Database>::TypeInfo {
        <String as Type<MySql>>::type_info()
    }
}

impl<'q> Encode<'q, MySql> for Email {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <String as Encode<MySql>>::encode_by_ref(&self.email, buf)
    }
}

impl<'r> Decode<'r, MySql> for Email {
    fn decode(
        value: <MySql as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let inner = <String as Decode<MySql>>::decode(value)?;
        Ok(Email { email: inner })
    }
}
