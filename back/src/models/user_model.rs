use actix_session::SessionExt;
use actix_web::FromRequest;
use serde::{Deserialize, Serialize};

use sqlx::{query, query_as};
use time::PrimitiveDateTime;

use std::error::Error;

use crate::{DB_POOL, services::auth_service::LoginCredential, shared::SQLable};

use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::{PasswordHash, SaltString, rand_core};

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub phone_number: Option<String>,
    pub verified_at: Option<PrimitiveDateTime>,
    pub admin: u8,
}

impl SQLable for User {
    async fn up() -> Result<(), Box<dyn Error>> {
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

    async fn down() -> Result<(), Box<dyn Error>> {
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
    pub async fn get(id: i32) -> Result<Option<Self>, Box<dyn Error>> {
        let user = query_as!(User, r#"SELECT * FROM users WHERE id=? LIMIT 1"#, id)
            .fetch_optional(&*DB_POOL)
            .await?;
        Ok(user)
    }
    pub async fn create(
        credential: &LoginCredential,
    ) -> Result<Result<i32, &'static str>, sqlx::Error> {
        let hashed_password = hash_password(credential.get_password());

        let db_response = query!(
            "INSERT INTO users (email,password) VALUES (?,?);",
            credential.get_email(),
            hashed_password
        )
        .execute(&*DB_POOL)
        .await;
        let err = match db_response {
            Ok(response) => return Ok(Ok(response.last_insert_id() as i32)),
            Err(err) => err,
        };
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.code().as_deref() == Some("1062") {
                // 1062 = Duplicate entry
                return Ok(Err("Email already exists"));
            }
        }
        // Pour les autres erreurs
        Err(err)
    }
    pub async fn get_from_credential(
        credential: &LoginCredential,
    ) -> Result<Option<Self>, Box<dyn Error>> {
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
}
impl FromRequest for User {
    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        let session = req.get_session();
    }
    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
    }
}
