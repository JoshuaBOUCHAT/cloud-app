use serde::{Deserialize, Serialize};

use chrono::NaiveDateTime;

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub phone_number: Option<String>,
    pub verified_at: Option<NaiveDateTime>,
}
