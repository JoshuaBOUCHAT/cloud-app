use actix_web::{HttpResponse, Responder, http::StatusCode};
use fancy_regex::Regex;
use serde::Serialize;
use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    auth::bearer_manager::Token,
    errors::{AppError, AppResult},
};

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type APIResponse = AppResult<JsonResponse>;

pub static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w{2,}$").unwrap());
pub static PASSWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^A-Za-z\d]).{8,}$").unwrap()
});

#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum JsonData {
    Token(Token),
    Message(String),
    Object(String),
}

#[repr(transparent)]
pub struct JsonResponseBuilder {
    status_code: StatusCode,
}
impl JsonResponseBuilder {
    pub fn token(self, token: Token) -> AppResult<JsonResponse> {
        Ok(JsonResponse {
            data: JsonData::Token(token),
            status_code: self.status_code,
        })
    }
    pub fn message(self, message: impl AsRef<str>) -> AppResult<JsonResponse> {
        Ok(JsonResponse {
            data: JsonData::Message(message.as_ref().to_string()),
            status_code: self.status_code,
        })
    }
    pub fn object(self, object: impl Serialize) -> AppResult<JsonResponse> {
        Ok(JsonResponse {
            data: JsonData::Object(serde_json::to_string(&object)?),
            status_code: self.status_code,
        })
    }
}
pub struct JsonResponse {
    data: JsonData,
    status_code: StatusCode,
}

impl JsonResponse {
    pub fn build(status_code: u16) -> Result<JsonResponseBuilder, AppError> {
        let status_code = StatusCode::from_u16(status_code).map_err(|err| {
            AppError::Internal(format!(
                "Error while parsing status code: {}   err:\n{err}",
                status_code
            ))
        })?;
        Ok(JsonResponseBuilder { status_code })
    }
    pub fn status(status_code: StatusCode) -> JsonResponseBuilder {
        JsonResponseBuilder { status_code }
    }
    pub fn ok() -> JsonResponseBuilder {
        JsonResponseBuilder {
            status_code: StatusCode::OK,
        }
    }
    pub fn not_found() -> JsonResponseBuilder {
        JsonResponseBuilder {
            status_code: StatusCode::NOT_FOUND,
        }
    }
    pub fn unauthorized() -> JsonResponseBuilder {
        JsonResponseBuilder {
            status_code: StatusCode::UNAUTHORIZED,
        }
    }
    pub fn token(token: Token) -> AppResult<JsonResponse> {
        Ok(JsonResponse {
            data: JsonData::Token(token),
            status_code: StatusCode::OK,
        })
    }
}
impl Responder for JsonResponse {
    type Body = actix_web::body::BoxBody;
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::build(self.status_code).json(self.data)
    }
}

pub fn get_now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
