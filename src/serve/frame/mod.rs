use std::time::Duration;
use serde::Serialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, time::timeout};
use self::{
    codec::{encode_u16, encode_u24, encode_u8},
    recv::RecvFrame, send::SendFrame,
};
use crate::{
    error::{proto_err, AppErr, ErrorExt, IoErr},
    serve::frame::{codec::{decode_u16, decode_u24, decode_u8}, recv::{ResponseFrame, RequestFrame}},
    utils::{new_bytes, Array},
};

mod codec;
pub mod recv;
pub mod send;

pub type Body = Array<u8>;

pub mod frame_type {

    pub const ACK: u8 = 0;
    pub const PING: u8 = 1;
    pub const PONG: u8 = 2;
    pub const REQ: u8 = 3;
    pub const RES: u8 = 4;
    pub const SIMPLE_REQ: u8 = 5;
    pub const SIMPLE_RES: u8 = 6;
    pub const NOTIFY: u8 = 7;
    pub const NOTIFY_ACK: u8 = 8;
}

pub fn make_type_seq(ft: u8, seq: u8) -> u16 {
    ((ft as u16) << 8) + (seq as u16)
}

/*
    head    2
    len     3
    seq     1
    type    1
    all     7
*/

pub const FRAME_HEAD_LEN: usize = 7;
pub const FRAME_HEAD: u16 = 0xE11E;
pub const RECV_TIMEOUT: Duration = Duration::from_secs(10);

pub struct BaseFrame {
    pub seq: u8,
}

impl BaseFrame {
    pub fn make(&self, ft: u8) -> Array<u8> {
        let mut buf = new_bytes(FRAME_HEAD_LEN);
        encode_u16(&mut buf, FRAME_HEAD);
        encode_u24(&mut buf[2..], FRAME_HEAD_LEN as u32);
        encode_u8(&mut buf[5..], self.seq);
        encode_u8(&mut buf[6..], ft);
        buf
    }
}


async fn read_body<IO: AsyncReadExt + Unpin>(len: usize, seq: u8, ft: u8, io: &mut IO) -> Result<RecvFrame, AppErr> {
    let body_len = len - FRAME_HEAD_LEN;
    let mut buf = new_bytes(body_len);
    io.read_exact(&mut buf).await?;

    let frame = match ft {
        frame_type::REQ => RecvFrame::Req(RequestFrame::new(seq, buf)?),
        frame_type::RES => RecvFrame::Res(ResponseFrame::new(seq, buf)?),

        frame_type::SIMPLE_REQ => RecvFrame::SimpleReq(RequestFrame::new(seq, buf)?),
        frame_type::SIMPLE_RES => RecvFrame::SimpleRes(ResponseFrame::new(seq, buf)?),

        frame_type::NOTIFY => RecvFrame::Notify(RequestFrame::new(seq, buf)?),
        frame_type::NOTIFY_ACK => RecvFrame::NotifyAck(RequestFrame::new(seq, buf)?),

        _ => return proto_err("invalid frame type"),
    };

    Ok(frame)
}

pub async fn read<IO: AsyncReadExt + Unpin>(io: &mut IO) -> Result<RecvFrame, AppErr> {
    let mut buf = new_bytes(FRAME_HEAD_LEN);
    timeout(RECV_TIMEOUT, io.read_exact(&mut buf))
        .await
        .wrap()??;

    let head = decode_u16(&buf);
    if head != FRAME_HEAD {
        return proto_err("frame head invalid");
    }

    let len = decode_u24(&buf[2..]) as usize;
    if len < FRAME_HEAD_LEN {
        return proto_err("frame len invalid");
    }

    let seq = decode_u8(&buf[5..]);
    let ft = decode_u8(&buf[6..]);

    let frame = match ft {
        frame_type::ACK => RecvFrame::Ack(BaseFrame { seq }),
        frame_type::PING => RecvFrame::Ping(BaseFrame { seq }),
        frame_type::PONG => RecvFrame::Pong(BaseFrame { seq }),
        frame_type::REQ
        | frame_type::RES
        | frame_type::SIMPLE_REQ
        | frame_type::SIMPLE_RES
        | frame_type::NOTIFY
        | frame_type::NOTIFY_ACK => read_body(len, seq, ft, io).await?,
        _ => return proto_err("invalid type"),
    };

    Ok(frame)
}

pub async fn write<IO: AsyncWriteExt + Unpin>(io: &mut IO, frame: &SendFrame) -> Result<(), IoErr> {
    let buf = frame.make();
    io.write_all(&buf).await?;
    Ok(())
} 

pub trait ToFrameBody {
    
    fn to_body(&self) -> Body;

    fn to_res(&self) -> Result<Body, AppErr>;

    fn to_vec(&self) -> Vec<u8>;
}

impl <T: Serialize> ToFrameBody for T {

    fn to_body(&self) -> Body {
        serde_cbor::to_vec(self).unwrap().into_boxed_slice()
    }

    fn to_vec(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).unwrap()
    }

    fn to_res(&self) -> Result<Body, AppErr> {
        Ok(self.to_body())
    }
}
