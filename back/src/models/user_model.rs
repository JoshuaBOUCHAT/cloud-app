use serde::{Deserialize, Serialize};

use chrono::NaiveDateTime;
use sqlx::{Executor, MySql, Pool, pool::PoolConnection, query};

use std::error::Error;

use async_trait::async_trait;

use crate::shared::SQLable;

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub phone_number: Option<String>,
    pub verified_at: Option<NaiveDateTime>,
}

impl SQLable for User {
    async fn up<'a>(conn: &'a Pool<MySql>) -> Result<(), Box<dyn Error>> {
        query!(
            r#"
            CREATE TABLE IF NOT EXISTS users(
                id INT AUTO_INCREMENT,
                email VARCHAR(100),
                password VARCHAR(100),
                phone_number VARCHAR(20),
                verified_at DATETIME,
                admin BOOLEAN,
                PRIMARY KEY(id)
            );
            "#
        )
        .execute(conn)
        .await?;
        Ok(())
    }

    async fn down<'a>(conn: &'a Pool<MySql>) -> Result<(), Box<dyn Error>> {
        query!(r#"DROP TABLE IF EXISTS users;"#)
            .execute(conn)
            .await?;
        Ok(())
    }
}
