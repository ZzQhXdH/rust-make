use ntex::web::{HttpServer, App};

use crate::{error::IoErr, config::WEB_ADDR};


mod req;
mod resp;
mod api;



pub async fn run() -> Result<(), IoErr> {

    let app = || {
        App::new()
        .configure(api::register)
    };
    println!("web serve:{}", WEB_ADDR);
    HttpServer::new(app)
    .bind(WEB_ADDR)?
    .run()
    .await
}
