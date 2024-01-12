use ntex::web::{ServiceConfig, self, post, Responder};
use serde::Deserialize;

use crate::{web::resp::Cbor, error::AppErr};

#[derive(Debug, Deserialize)]
struct CreateReq {

}

#[post("/create")]
async fn create(req: Cbor<CreateReq>) -> Result<&'static str, AppErr> {

    todo!()
}

pub fn register(cfg: &mut ServiceConfig) {
    let scope = web::scope("/device")
    ;
    cfg.service(scope);
}

