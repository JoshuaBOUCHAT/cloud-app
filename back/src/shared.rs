use std::error::Error;

pub trait SQLable {
    async fn up() -> Result<(), Box<dyn Error>>;
    async fn down() -> Result<(), Box<dyn Error>>;
}
