use sqlx::{Executor, MySql, Pool, mysql, pool::PoolConnection};

use async_trait::async_trait;
use std::error::Error;

pub trait SQLable {
    async fn up<'a>(conn: &'a Pool<MySql>) -> Result<(), Box<dyn Error>>;
    async fn down<'a>(conn: &'a Pool<MySql>) -> Result<(), Box<dyn Error>>;
}
