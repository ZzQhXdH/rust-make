use ntex::web::{App, HttpServer};

use crate::{config::WEB_ADDR, error::IoErr};

mod api;
mod req;
mod resp;

pub async fn run() -> Result<(), IoErr> {
    let app = || App::new().configure(api::register);
    println!("web serve:{}", WEB_ADDR);
    HttpServer::new(app).bind(WEB_ADDR)?.run().await
}
