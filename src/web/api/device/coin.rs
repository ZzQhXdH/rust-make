use crate::store;
use crate::web::resp::{new_cbor, Cbor, CborRes};
use ntex::web::post;
use ntex::web::{self, ServiceConfig};
use serde::Deserialize;

#[post("/get")]
async fn get(device_id: Cbor<i64>) -> CborRes<store::coin::TableCoin> {
    use store::coin::*;
    let info = get(*device_id).await?;
    new_cbor(info)
}

#[post("/get_info")]
async fn get_info(device_id: Cbor<i64>) -> CborRes<store::coin::TableCoinInfos> {
    let infos = store::coin::get_info(*device_id).await?;
    new_cbor(infos)
}

#[derive(Debug, Deserialize)]
struct TypeMaskReq {
    device_id: i64,
    mask: u32,
}

#[post("/set_mask")]
async fn set_mask(req: Cbor<TypeMaskReq>) -> CborRes<()> {
    store::coin::set_type_mask(req.device_id, req.mask).await?;
    new_cbor(())
}

pub fn register(cfg: &mut ServiceConfig) {
    let scope = web::scope("/coin")
        .service(get)
        .service(get_info)
        .service(set_mask);
    cfg.service(scope);
}
