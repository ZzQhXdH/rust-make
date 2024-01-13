use super::proto::ReadFrame;
use crate::{
    error::{proto_err, AppErr},
    serve::proto::{HEAD, MIN_LEN, TYPE_NOTIFY, TYPE_PING, TYPE_PONG, TYPE_REQ, TYPE_RES},
    utils::{
        codec::{decode_u16, decode_u8},
        new_bytes,
    },
};
use tokio::io::AsyncReadExt;

// req/res   notify
pub const SEQ_CMD_INDEX: usize = 0;

// req/res
pub const CMD_INDEX: usize = 1;

pub async fn read_frame<R: AsyncReadExt + Unpin>(r: &mut R) -> Result<ReadFrame, AppErr> {
    let mut buf = new_bytes(MIN_LEN);
    r.read_exact(&mut buf).await?;

    let head = decode_u16(&buf);
    if head != HEAD {
        return proto_err("proto head invalid");
    }

    let len = decode_u16(&buf[2..]) as usize;
    if len < MIN_LEN {
        return proto_err("proto len invalid");
    }

    let proto_type = decode_u8(&buf[4..]);

    if len == MIN_LEN {
        match proto_type {
            TYPE_PING => return Ok(ReadFrame::Ping),
            TYPE_PONG => return Ok(ReadFrame::Pong),
            _ => return proto_err("invalid type"),
        };
    }

    let mut body = new_bytes(len - MIN_LEN);
    r.read_exact(&mut body).await?;

    match proto_type {
        TYPE_REQ => Ok(ReadFrame::Req(body)),
        TYPE_RES => Ok(ReadFrame::Resp(body)),
        TYPE_NOTIFY => Ok(ReadFrame::Notify(body)),
        _ => proto_err("invalid type2"),
    }
}
