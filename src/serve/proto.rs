use crate::utils::{
    codec::{encode_u16, encode_u8, memcpy},
    new_bytes, Array,
};
use serde::Serialize;

pub const HEAD: u16 = 0xE11E;
pub const MIN_LEN: usize = 5;

/*
 * Proto
 * E1 1E LEN_H LEN_L TYPE ...
 */
pub const TYPE_PING: u8 = 1;
pub const TYPE_PONG: u8 = 2;
pub const TYPE_REQ: u8 = 3;
pub const TYPE_RES: u8 = 4;
pub const TYPE_NOTIFY: u8 = 5;

pub enum ReadFrame {
    Ping,
    Pong,
    Req(Array<u8>),
    Resp(Array<u8>),
    Notify(Array<u8>),
}

pub fn make_ping() -> Array<u8> {
    let mut buf = new_bytes(MIN_LEN);
    encode_u16(&mut buf, HEAD);
    encode_u16(&mut buf[2..], 5);
    encode_u8(&mut buf[4..], TYPE_PING);
    buf
}

pub fn make_pong() -> Array<u8> {
    let mut buf = new_bytes(MIN_LEN);
    encode_u16(&mut buf, HEAD);
    encode_u16(&mut buf[2..], 5);
    encode_u8(&mut buf[4..], TYPE_PONG);
    buf
}

pub fn make_req<T: Serialize>(seq: u8, cmd: u8, value: T) -> Array<u8> {
    let body = serde_cbor::to_vec(&value).unwrap();
    let len = MIN_LEN + 2 + body.len();
    let mut buf = new_bytes(len);

    encode_u16(&mut buf, HEAD);
    encode_u16(&mut buf[2..], len as u16);
    encode_u8(&mut buf[4..], TYPE_REQ);
    encode_u8(&mut buf[5..], seq);
    encode_u8(&mut buf[6..], cmd);
    memcpy(&mut buf[7..], &body);

    buf
}

pub fn make_res_buf(seq: u8, cmd: u8, ec: u8, body: Array<u8>) -> Array<u8> {
    let len = MIN_LEN + 3 + body.len();
    let mut buf = new_bytes(len);

    encode_u16(&mut buf, HEAD);
    encode_u16(&mut buf[2..], len as u16);
    encode_u8(&mut buf[4..], TYPE_RES);
    encode_u8(&mut buf[5..], seq);
    encode_u8(&mut buf[6..], cmd);
    encode_u8(&mut buf[7..], ec);
    memcpy(&mut buf[8..], &body);

    buf
}

pub fn make_res<T: Serialize>(seq: u8, cmd: u8, ec: u8, value: T) -> Array<u8> {
    let body = serde_cbor::to_vec(&value).unwrap();
    let len = MIN_LEN + 3 + body.len();
    let mut buf = new_bytes(len);

    encode_u16(&mut buf, HEAD);
    encode_u16(&mut buf[2..], len as u16);
    encode_u8(&mut buf[4..], TYPE_RES);
    encode_u8(&mut buf[5..], seq);
    encode_u8(&mut buf[6..], cmd);
    encode_u8(&mut buf[7..], ec);
    memcpy(&mut buf[8..], &body);

    buf
}

pub fn make_notify<T: Serialize>(cmd: u8, value: T) -> Array<u8> {
    let body = serde_cbor::to_vec(&value).unwrap();
    let len = MIN_LEN + 1 + body.len();
    let mut buf = new_bytes(len);

    encode_u16(&mut buf, HEAD);
    encode_u16(&mut buf[2..], len as u16);
    encode_u8(&mut buf[4..], TYPE_RES);
    encode_u8(&mut buf[5..], cmd);
    memcpy(&mut buf[6..], &body);

    buf
}
