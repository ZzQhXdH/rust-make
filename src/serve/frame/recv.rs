use serde::Deserialize;
use crate::{utils::Array, error::{AppErr, ErrInfo, proto_err}};
use super::BaseFrame;



pub enum RecvFrame {
    Ack(BaseFrame),
    Ping(BaseFrame),
    Pong(BaseFrame),
    Req(RequestFrame),
    Res(ResponseFrame),
    SimpleReq(RequestFrame),
    SimpleRes(ResponseFrame),
    Notify(RequestFrame),
    NotifyAck(RequestFrame)
}

impl RecvFrame {

    pub fn ack(self) -> Result<BaseFrame, AppErr> {
        if let RecvFrame::Ack(f) = self {
            Ok(f)
        } else {
            proto_err("invalid ack")
        }
    }

    pub fn ping(self) -> Result<BaseFrame, AppErr> {
        if let RecvFrame::Ping(f) = self {
            Ok(f)
        } else {
            proto_err("invalid ping")
        }
    }

    pub fn pong(self) -> Result<BaseFrame, AppErr> {
        if let RecvFrame::Pong(f) = self {
            Ok(f)
        } else {
            proto_err("invalid pong")
        }
    }

    pub fn req(self) -> Result<RequestFrame, AppErr> {
        if let RecvFrame::Req(f) = self {
            Ok(f)
        } else {
            proto_err("invalid req")
        }
    }

    pub fn res(self) -> Result<ResponseFrame, AppErr> {
        if let RecvFrame::Res(f) = self {
            Ok(f)
        } else {
            proto_err("invalid res")
        }
    }

    pub fn simple_req(self) -> Result<RequestFrame, AppErr> {
        if let RecvFrame::SimpleReq(f) = self {
            Ok(f)
        } else {
            proto_err("invalid simple req")
        }
    }

    pub fn simple_res(self) -> Result<ResponseFrame, AppErr> {
        if let RecvFrame::SimpleRes(f) = self {
            Ok(f)
        } else {
            proto_err("invalid simple res")
        }
    }

    pub fn parse<'a, T: Deserialize<'a>>(&'a self) -> Result<T, AppErr> {
        match self {
            RecvFrame::Ack(_) | RecvFrame::Ping(_) | RecvFrame::Pong(_) => proto_err("parse invalid type"),
            RecvFrame::Req(r) => r.parse(),
            RecvFrame::Res(r) => r.parse(),
            RecvFrame::SimpleReq(r) => r.parse(),
            RecvFrame::SimpleRes(r) => r.parse(),
            RecvFrame::Notify(r) => r.parse(),
            RecvFrame::NotifyAck(r) => r.parse(),
        }
    }
}
pub struct RequestFrame {
    pub seq: u8,
    body: Array<u8>,
}

impl RequestFrame {

    pub fn new(seq: u8, body: Array<u8>) -> Result<Self, AppErr> {
        let len = body.len();
        if len < 1 {
            return proto_err("req len < 1")
        }
        let frame = RequestFrame {
            seq,
            body,
        };
        Ok(frame)
    }

    pub fn cmd(&self) -> u8 {
        self.body[0]
    }

    pub fn parse<'a, R: Deserialize<'a>>(&'a self) -> Result<R, AppErr> {
        let v = serde_cbor::from_slice(&self.body[1..])?;
        Ok(v)
    }
}

pub struct ResponseFrame {
    pub seq: u8,
    body: Array<u8>,
}

impl ResponseFrame {

    pub fn new(seq: u8, body: Array<u8>) -> Result<Self, AppErr> {
        let len = body.len();
        if len < 2 {
            return proto_err("res len < 2")
        }
        let frame = ResponseFrame {
            seq,
            body
        };
        Ok(frame)
    }

    pub fn cmd(&self) -> u8 {
        self.body[0]
    }

    pub fn parse<'a, R: Deserialize<'a>>(&'a self) -> Result<R, AppErr> {
        let ec = self.body[1];
        if ec == 0 {
            let v = serde_cbor::from_slice::<R>(&self.body[2..])?;
            Ok(v)
        } else {
            let e = serde_cbor::from_slice::<ErrInfo>(&self.body[2..])?;
            Err(AppErr::Custom(e))
        }
    }
}
