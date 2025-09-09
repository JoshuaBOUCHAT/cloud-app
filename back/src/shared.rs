use sqlx::Executor;
use sqlx::mysql::MySqlPoolOptions;

use std::error::Error;

pub trait SQLable {
    pub async fn up<'e, 'c: 'e, E>(executor: E) -> Result<(), Box<dyn Error>>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>;
    fn down() {}
}
