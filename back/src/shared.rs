use sqlx::Executor;
use sqlx::mysql::MySqlPoolOptions;

use std::error::Error;

pub trait SQLable {
    type DB;
    async fn up<'c, E>(executor: E) -> Result<(), Box<dyn Error>>
    where
        E: Executor<'c, Database = Self::DB>;
    fn down() {}
}
