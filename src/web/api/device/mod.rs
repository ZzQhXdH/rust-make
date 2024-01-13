use crate::{
    store,
    utils::Array,
    web::resp::{new_cbor, Cbor, CborRes},
};
use ntex::web::{self, post, ServiceConfig};
use serde::Deserialize;

mod bill;
mod coin;

#[derive(Debug, Deserialize)]
struct CreateReq {
    name: String,
    address: String,
    mac_addr: String,
}

#[post("/create")]
async fn create(req: Cbor<CreateReq>) -> CborRes<i64> {
    use store::device::*;
    let id = create_by(&req.mac_addr, &req.name, &req.address).await?;
    new_cbor(id)
}

#[post("/get")]
async fn get_by_id(id: Cbor<i64>) -> CborRes<store::device::TableDevice> {
    use store::device::*;
    let info = get(*id).await?;
    new_cbor(info)
}

#[post("/select")]
async fn select() -> CborRes<Array<store::device::TableDevice>> {
    use store::device::*;
    let infos = select().await?;
    new_cbor(infos)
}

#[post("/delete")]
async fn delete(id: Cbor<i64>) -> CborRes<()> {
    use store::device::*;
    delete(*id).await?;
    new_cbor(())
}

#[derive(Debug, Deserialize)]
struct UpdateReq {
    id: i64,
    mac_addr: Option<String>,
    name: Option<String>,
    address: Option<String>,
}

#[post("/update")]
async fn update(req: Cbor<UpdateReq>) -> CborRes<()> {
    use store::device::*;

    set_mac_addr(req.id, req.mac_addr.as_deref()).await?;
    set_name(req.id, req.name.as_deref()).await?;
    set_address(req.id, req.address.as_deref()).await?;

    new_cbor(())
}

pub fn register(cfg: &mut ServiceConfig) {
    let scope = web::scope("/device")
        .service(create)
        .service(get_by_id)
        .service(select)
        .service(update)
        .configure(coin::register)
        .configure(bill::register);
    cfg.service(scope);
}
