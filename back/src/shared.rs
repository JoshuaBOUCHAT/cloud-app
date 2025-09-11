use std::error::Error;

#[allow(async_fn_in_trait)]
pub trait SQLable {
    async fn up() -> Result<(), Box<dyn Error>>;
    async fn down() -> Result<(), Box<dyn Error>>;
}
