use error::AppErr;

mod config;
mod error;
mod serve;
mod store;
mod utils;
mod web;

#[ntex::main]
async fn main() -> Result<(), AppErr> {

    config::init().await?;
    store::sql_init().await?;
    serve::run().await;
    web::run().await?;


    Ok(())
}
