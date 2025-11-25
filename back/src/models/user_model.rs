use serde::{Deserialize, Serialize};

use sqlx::{query, query_as, query_scalar};
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::{
    DB_POOL,
    auth::{
        auth_extractor::TryFromClaim,
        auth_models::{
            claims::Claims,
            credential::{Email, LoginCredential},
            token::{Token, TokenAble},
        },
    },
    constants::messages::{EMAIL_ALREADY_EXIST, USER_NOT_FOUND},
    errors::{AppError, AppResult},
    utils::redis_utils::{redis_get, redis_set_ex},
};

use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::{PasswordHash, SaltString, rand_core};

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub email: Email,
    pub password: String,
    pub phone_number: Option<String>,
    pub verified_at: Option<PrimitiveDateTime>,
    pub admin: u8,
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
        if let Some(user) = redis_get(&id).await? {
            return Ok(Some(user));
        }

        let maybe_user = query_as!(User, r#"SELECT * FROM users WHERE id=? LIMIT 1"#, id)
            .fetch_optional(&*DB_POOL)
            .await?;
        if let Some(user) = &maybe_user {
            let _ = redis_set_ex(&format!("user:{}", user.id), user, 3600).await;
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
                let maybe_user = User::get(user_id).await?;
                if let Some(user) = maybe_user {
                    let _ = redis_set_ex(&format!("user:{}", user.id), &user, 3600).await;
                }

                return Ok(user_id);
            }

            Err(err) => err,
        };

        if let sqlx::Error::Database(db_err) = &err {
            let err = db_err.downcast_ref::<sqlx::mysql::MySqlDatabaseError>();
            if err.number() == 1062 {
                // Duplicate entry
                return Err(AppError::Conflict(EMAIL_ALREADY_EXIST.to_string()));
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
    pub async fn is_valide_user(id: i32) -> AppResult<bool> {
        Ok(Self::get(id).await?.is_some_and(|u| u.is_verified()))
    }
    ///Only return a token if the user have been verified
    pub async fn get_token(id: i32) -> AppResult<Option<Token>> {
        let maybe_user = Self::get(id).await?;
        let Some(user) = maybe_user else {
            return Err(AppError::Internal(USER_NOT_FOUND.to_string()));
        };

        if user.is_verified() {
            let new_claims = Claims::new_user_claim(id);
            return Ok(Some(new_claims.encode()?));
        } else {
            Ok(None)
        }
    }
    pub async fn get_claim(id: i32) -> AppResult<Option<Claims>> {
        let Some(user) = Self::get(id).await? else {
            return Err(AppError::Internal(USER_NOT_FOUND.to_string()));
        };

        if user.is_verified() {
            return Ok(Some(Claims::new_user_claim(id)));
        } else {
            Ok(None)
        }
    }
    ///This function do not check if the for password so the id should not be use where security is needed
    pub async fn get_user_id_from_mail(email: &Email) -> AppResult<Option<i32>> {
        let maybe_user_id = query_scalar!(
            r#"SELECT id FROM users WHERE email=? LIMIT 1"#,
            email.as_ref()
        )
        .fetch_optional(&*DB_POOL)
        .await?;
        Ok(maybe_user_id)
    }

    pub fn is_admin(&self) -> bool {
        self.admin != 0
    }
    pub fn is_verified(&self) -> bool {
        self.verified_at.is_some()
    }
}

use async_trait::async_trait;

#[async_trait]
impl TryFromClaim for User {
    async fn try_from_claim(claims: &Claims) -> Result<Self, actix_web::Error> {
        let user = Self::get(claims.user_id)
            .await?
            .ok_or_else(|| AppError::Internal(USER_NOT_FOUND.to_string()))?;
        Ok(user)
    }
}
