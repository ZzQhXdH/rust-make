use error::AppErr;


mod error;
mod serve;
mod web;
mod store;
mod config;
mod utils;

#[ntex::main]
async fn main() -> Result<(), AppErr> {
    config::init().await?;
    store::sql_init().await?;
    web::run().await?;

    Ok(())
}
