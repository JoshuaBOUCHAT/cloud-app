use serde::{Deserialize, Serialize};

use sqlx::{MySql, Pool, query, query_as};
use time::PrimitiveDateTime;

use std::error::Error;

use crate::{DB_POOL, shared::SQLable};

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
                email VARCHAR(100) NOT NULL,
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

impl User {
    pub async fn get(id: i32) -> Result<Option<Self>, Box<dyn Error>> {
        let user = query_as!(User, r#"SELECT * FROM users WHERE id=? LIMIT 1"#, id)
            .fetch_optional(&*DB_POOL)
            .await?;
        Ok(user)
    }
    pub async fn create(email: &str, password: &str) -> Result<i32, Box<dyn Error>> {
        let res = query!(
            "INSERT INTO users (email,password) VALUES (?,?);",
            email,
            password
        )
        .execute(&*DB_POOL)
        .await?;
        Ok(res.last_insert_id() as i32)
    }
}
