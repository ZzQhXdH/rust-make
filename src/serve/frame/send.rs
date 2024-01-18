use serde::Serialize;
use crate::{utils::{Array, new_bytes}, serve::frame::{codec::{encode_u16, encode_u24, encode_u8}, FRAME_HEAD}, error::{AppErr, serial_to_vec}};
use super::{codec::memcpy, frame_type, FRAME_HEAD_LEN, BaseFrame, Body};


const PROTO_REQ_HEAD_LEN: usize = FRAME_HEAD_LEN + 1; // cmd
const PROTO_RES_HEAD_LEN: usize = FRAME_HEAD_LEN + 2; // cmd + ec;


pub enum SendFrame {

    Ack(BaseFrame),
    Ping(BaseFrame),
    Pong(BaseFrame),
    Req(RequestFrame),
    SimpleReq(RequestFrame),
    Res(ResponseFrame),
    SimpleRes(ResponseFrame),
    Notify(RequestFrame),
    NotifyAck(RequestFrame),
}

impl SendFrame {

    pub fn make(&self) -> Array<u8> {
        match self {
            Self::Ack(v) => v.make(frame_type::ACK),
            Self::Ping(v) => v.make(frame_type::PING),
            Self::Pong(v) => v.make(frame_type::PONG),
            Self::Req(v) => v.make(frame_type::REQ),
            Self::SimpleReq(v) => v.make(frame_type::SIMPLE_REQ),
            Self::Res(v) => v.make(frame_type::RES),
            Self::SimpleRes(v) => v.make(frame_type::SIMPLE_RES),
            Self::Notify(v) => v.make(frame_type::NOTIFY),
            Self::NotifyAck(v) => v.make(frame_type::NOTIFY_ACK),
        }
    }
}

pub struct ResponseFrame {

    pub seq: u8,
    pub cmd: u8,
    pub ec: u8,
    pub body: Option<Body>,
}

impl ResponseFrame {

    pub fn new<T: Serialize>(seq: u8, cmd: u8, value: Result<T, AppErr>) -> Self {
        let ec = value.as_ref().map_or(1, |_| 0);
        Self { 
            seq,
            cmd, 
            ec,
            body: Some( serial_to_vec(value).into_boxed_slice() ), 
        }
    }

    pub fn new_body(seq: u8, cmd: u8, value: Result<Body, AppErr>) -> Self {
        let ec = value.as_ref().map_or(1, |_| 0);
        let body = match value {
            Ok(v) => v,
            Err(e) => e.serial_to_vec().into_boxed_slice(),
        };
        Self { 
            seq,
            cmd, 
            ec,
            body: Some(body)
        }
    }

    pub fn new_ok<T: Serialize>(seq: u8, cmd: u8, value: &T) -> Self {
        Self { 
            seq, 
            cmd, 
            ec: 0, 
            body: Some(serde_cbor::to_vec(value).unwrap().into_boxed_slice()),
        }
    }

    pub fn new_with_err(seq: u8, cmd: u8, value: Option<AppErr>) -> Self {
        let ec = value.as_ref().map_or(0, |_| 1);
        let body = value.map(|e| e.serial_to_vec().into_boxed_slice());
        Self {
            seq,
            cmd,
            ec,
            body
        }
    }

    pub fn make(&self, ft: u8) -> Array<u8> {
        let len = self.body.as_ref().map_or(0, |v| v.len()) + PROTO_RES_HEAD_LEN;
        let mut buf = new_bytes(len);
        encode_u16(&mut buf, FRAME_HEAD);
        encode_u24(&mut buf[2..], len as u32);
        encode_u8(&mut buf[5..], self.seq);
        encode_u8(&mut buf[6..], ft);
        encode_u8(&mut buf[7..], self.cmd);
        encode_u8(&mut buf[8..], self.ec);
        if let Some(body) = &self.body {
            memcpy(&mut buf[PROTO_RES_HEAD_LEN..], body);
        }
        buf
    }
}

pub struct RequestFrame {
    pub seq: u8,
    pub cmd: u8,
    pub body: Option<Array<u8>>,
}

impl RequestFrame {

    pub fn new<T: Serialize>(seq: u8, cmd: u8, value: &T) -> Self {
        let body = serde_cbor::to_vec(value).unwrap();
        Self {
            seq,
            cmd,
            body: Some(body.into_boxed_slice()),
        }
    }

    pub fn new_with_body(seq: u8, cmd: u8, value: Option<Array<u8>>) -> Self {
        Self {
            seq,
            cmd,
            body: value,
        }
    }

    pub fn make(&self, ft: u8) -> Array<u8> {
        let len = self.body.as_ref().map_or(0, |v| v.len()) + PROTO_REQ_HEAD_LEN;
        let mut buf = new_bytes(len);
        encode_u16(&mut buf, FRAME_HEAD);
        encode_u24(&mut buf[2..], len as u32);
        encode_u8(&mut buf[5..], self.seq);
        encode_u8(&mut buf[6..], ft);
        encode_u8(&mut buf[7..], self.cmd);
        if let Some(body) = &self.body {
            memcpy(&mut buf[PROTO_REQ_HEAD_LEN..], body);
        }
        buf
    }
}



