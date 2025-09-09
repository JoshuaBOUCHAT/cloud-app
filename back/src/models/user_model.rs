use serde::{Deserialize, Serialize};

use chrono::NaiveDateTime;
use sqlx::{MySql, Pool, query};

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
    type DB = Pool<MySql>;
    async fn up<'c, E>(executor: E) -> Result<(), Box<dyn std::error::Error>>
    where
        E: sqlx::Executor<'c, Database = Self::DB>,
    {
        query!(
            "CREATE TABLE users(
            id INT AUTO_INCREMENT,
            email VARCHAR(100) ,
            password VARCHAR(100) ,
            phone_number VARCHAR(20) ,
            verified_at DATETIME,
            admin BOOLEAN,
            PRIMARY KEY(id)
         );"
        ).
    }

    fn down() {}
}
