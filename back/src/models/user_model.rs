use actix_session::SessionExt;
use actix_web::FromRequest;
use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};

use sqlx::{query, query_as};
use time::{OffsetDateTime, PrimitiveDateTime};

use std::{error::Error, pin::Pin};

use actix_web::Error as ActixError;

use crate::{
    DB_POOL,
    errors::{AppError, AppResult},
    services::auth_service::LoginCredential,
    shared::SQLable,
    utils::redis_utils::{redis_get, redis_set_ex},
};

use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::{PasswordHash, SaltString, rand_core};

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub phone_number: Option<String>,
    pub verified_at: Option<PrimitiveDateTime>,
    pub admin: u8,
}

impl ToRedisArgs for User {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        let json = serde_json::to_string(self).expect("User serialization failed");
        out.write_arg(json.as_bytes());
    }
}

impl FromRedisValue for User {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        let s: String = redis::from_redis_value(v)?;
        let user: User = serde_json::from_str(&s).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Failed to deserialize User from JSON",
                e.to_string(),
            ))
        })?;
        Ok(user)
    }
}

impl SQLable for User {
    async fn up() -> AppResult<()> {
        query!(
            r#"
                CREATE TABLE IF NOT EXISTS users(
                    id INT AUTO_INCREMENT,
                    email VARCHAR(100) NOT NULL UNIQUE,
                    password VARCHAR(100) NOT NULL,
                    phone_number VARCHAR(20),
                    verified_at DATETIME,
                    admin TINYINT(1) UNSIGNED NOT NULL DEFAULT 0,
                    PRIMARY KEY(id)
                );
                "#
        )
        .execute(&*DB_POOL)
        .await?;
        Ok(())
    }

    async fn down() -> AppResult<()> {
        query!(r#"DROP TABLE IF EXISTS users;"#)
            .execute(&*DB_POOL)
            .await?;
        Ok(())
    }
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    password_hash.to_string() // contient hash + salt + paramètres encodés
}

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

impl User {
    pub async fn get(id: i32) -> AppResult<Option<Self>> {
        if let Some(user) = redis_get(id).await? {
            return Ok(Some(user));
        }

        let maybe_user = query_as!(User, r#"SELECT * FROM users WHERE id=? LIMIT 1"#, id)
            .fetch_optional(&*DB_POOL)
            .await?;

        if let Some(ref user) = maybe_user {
            // clone uniquement pour le spawn
            let user_to_cache = user.clone();
            actix_rt::spawn(async move {
                let _ =
                    redis_set_ex(format!("user:{}", user_to_cache.id), user_to_cache, 3600).await;
            });
        }

        Ok(maybe_user)
    }
    pub async fn create(credential: &LoginCredential) -> AppResult<i32> {
        let hashed_password = hash_password(credential.get_password());

        let db_response = query!(
            "INSERT INTO users (email,password) VALUES (?,?);",
            credential.get_email(),
            hashed_password
        )
        .execute(&*DB_POOL)
        .await;
        let err = match db_response {
            Ok(response) => {
                let user_id = response.last_insert_id() as i32;
                actix_rt::spawn(async move {
                    if let Ok(Some(user)) = Self::get(user_id).await {
                        let _ = redis_set_ex(format!("user:{}", user.id), user, 3600).await;
                    }
                });
                return Ok(user_id);
            }
            Err(err) => err,
        };
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.code().as_deref() == Some("1062") {
                // 1062 = Duplicate entry
                return Err(AppError::Conflict("Email already exists".to_string()));
            }
        }
        // Pour les autres erreurs
        Err(err.into())
    }
    pub async fn get_from_credential(credential: &LoginCredential) -> AppResult<Option<Self>> {
        let maybe_user = query_as!(
            User,
            r#"SELECT * FROM users WHERE email=? LIMIT 1"#,
            credential.get_email(),
        )
        .fetch_optional(&*DB_POOL)
        .await?;
        let Some(user) = maybe_user else {
            return Ok(None);
        };
        if !verify_password(credential.get_password(), &user.password) {
            return Ok(None);
        }

        Ok(Some(user))
    }
    pub async fn verify_user(user_id: i32) -> AppResult<()> {
        // on récupère le timestamp actuel
        let now = OffsetDateTime::now_utc();

        // mise à jour de l'utilisateur
        query!(
            r#"
            UPDATE users
            SET verified_at = ?
            WHERE id = ?
            "#,
            now,
            user_id
        )
        .execute(&*DB_POOL)
        .await?;

        Ok(())
    }

    async fn actix_response(user_id: i32) -> Result<Self, ActixError> {
        let maybe_user = match Self::get(user_id).await {
            Ok(maybe_user) => maybe_user,
            Err(_) => {
                return Err(actix_web::error::ErrorInternalServerError(
                    "An error ocurs sorry !",
                ));
            }
        };
        let Some(user) = maybe_user else { todo!() };
        if user.verified_at.is_none() {
            return Err(actix_web::error::ErrorUnauthorized(
                "The user account has not been verified",
            ));
        }
        Ok(user)
    }
}

impl FromRequest for User {
    type Error = ActixError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let session = req.get_session();

        Box::pin(async move {
            // récupérer l'id depuis la session
            let Some(user_id) = session
                .get::<i32>("user_id")
                .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid session"))?
            else {
                return Err(actix_web::error::ErrorUnauthorized("User not logged in"));
            };

            // utiliser ta fonction async
            let user = User::actix_response(user_id).await?;
            Ok(user)
        })
    }
}
