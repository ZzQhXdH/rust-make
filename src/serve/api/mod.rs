use std::{time::Duration, sync::atomic::{AtomicU32, Ordering}, net::SocketAddr};

use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, time};

use super::{conn::SharedConn, frame::{read, recv::RequestFrame, Body}};
use crate::{
    error::{proto_err, AppErr, ErrorExt},
    store,
    serve::frame::{write, send::{SendFrame, ResponseFrame}, BaseFrame},
};

const CMD_LOGIN: u8 = 0x01;

#[derive(Debug, Deserialize)]
struct CoinInfo {
    model: String,
    version: String,
    serial_number: String,
}

#[derive(Debug, Deserialize)]
struct BillInfo {
    model: String,
    version: String,
    serial_number: String,
}

#[derive(Debug, Deserialize)]
struct LoginReq {
    mac_addr: String,
    app_version: String,
    mcu_version: Option<String>,
    coin_info: Option<CoinInfo>,
    bill_info: Option<BillInfo>,
}


#[derive(Debug, Serialize)]
pub struct ConnInfo {
    pub addr: SocketAddr,
    pub mac_addr: String,
    pub id: i64,
    pub ping_count: AtomicU32,
}

impl ConnInfo {

    pub fn ping(&self) {
        self.ping_count.fetch_add(1, Ordering::SeqCst);
    }


}

pub async fn wait_login(stream: &mut TcpStream, addr: SocketAddr) -> Result<ConnInfo, AppErr> {
    let frame = time::timeout(Duration::from_secs(10), read(stream)).await.wrap()?.wrap()?;
    let req_frame = frame.req()?;
    let seq = req_frame.seq;
    let cmd = req_frame.cmd();
    if cmd != CMD_LOGIN {
        return proto_err("not login");
    }
    let req: LoginReq = req_frame.parse()?;

    write(stream, &SendFrame::Ack(BaseFrame{ seq })).await?;
    
    let id = login(&req).await?;

    write(stream, &SendFrame::Res(ResponseFrame::new(seq, cmd, Ok(id)))).await?;

    let info = ConnInfo {
        id,
        mac_addr: req.mac_addr,
        ping_count: AtomicU32::new(0),
        addr,
    };

    Ok(info)

}

async fn login(req: &LoginReq) -> Result<i64, AppErr> {
    use store::*;

    let id = device::create_if_not_exists(&req.mac_addr).await?;
    device::set_app_version(id, &req.app_version).await?;

    if let Some(mcu_version) = &req.mcu_version {
        device::set_muc_version(id, mcu_version).await?;
    }
    if let Some(coin) = &req.coin_info {
        coin::update(id, &coin.model, &coin.version, &coin.serial_number).await?;
    }
    if let Some(bill) = &req.bill_info {
        bill::update(id, &bill.model, &bill.version, &bill.serial_number).await?;
    }

    Ok(id)
}

async fn dispatch(_frame: RequestFrame) -> Result<Body, AppErr> {

    todo!()
}

pub async fn handle_req(conn: SharedConn, frame: RequestFrame) {

    let cmd = frame.cmd();
    let seq = frame.seq;
    let result: Result<Body, AppErr> = match cmd {
        _ => proto_err("invalid cmd")
    };
    _ = conn.write(SendFrame::Res(ResponseFrame::new_body(seq, cmd, result)));
}

