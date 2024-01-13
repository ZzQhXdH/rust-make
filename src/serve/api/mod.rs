use serde::Deserialize;

use super::{conn::SharedConn, proto::make_res_buf};
use crate::{
    error::{proto_err, AppErr, ErrInfo, ErrorExt},
    store,
    utils::Array,
};

const CMD_LOGIN: u8 = 0x01;

#[derive(Debug, Deserialize)]
struct CoinInfo {
    model: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct BillInfo {
    model: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct LoginReq {
    mac_addr: String,
    app_version: String,
    mcu_version: Option<String>,
    coin_info: Option<CoinInfo>,
    bill_info: Option<BillInfo>,
}

async fn login(buf: &[u8]) -> Result<Array<u8>, AppErr> {
    use store::*;

    let req: LoginReq = serde_cbor::from_slice(buf)?;

    let id = device::create_if_not_exists(&req.mac_addr).await?;
    device::set_app_version(id, &req.app_version).await?;

    if let Some(mcu_version) = &req.mcu_version {
        device::set_muc_version(id, mcu_version).await?;
    }
    if let Some(coin) = &req.coin_info {
        coin::update(id, &coin.model, &coin.version).await?;
    };
    if let Some(bill) = &req.bill_info {
        bill::update(id, &bill.model, &bill.version).await?;
    }

    todo!()
}

pub async fn handle_resp(conn: SharedConn, frame: Array<u8>) {
    let len = frame.len();
    if len < 2 {
        return;
    }
    let seq = frame[0];
    let cmd = frame[1];

    let ret = match cmd {
        CMD_LOGIN => login(&&frame[2..]).await,
        _ => proto_err::<Array<u8>>("invalid cmd"),
    };

    match ret {
        Ok(v) => {
            conn.write(make_res_buf(seq, cmd, 0, v)).print_if_err();
        }
        Err(e) => {
            conn.write(make_res_buf(seq, cmd, 1, serial_err(e)))
                .print_if_err();
        }
    };
}

fn serial_err(err: AppErr) -> Array<u8> {
    let body = match err {
        AppErr::Custom(info) => serde_cbor::to_vec(&info).unwrap(),
        _ => {
            let resp = ErrInfo {
                err_code: -1,
                err_msg: err.to_string(),
            };
            serde_cbor::to_vec(&resp).unwrap()
        }
    };
    body.into_boxed_slice()
}
