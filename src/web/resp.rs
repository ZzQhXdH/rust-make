use std::ops::{Deref, DerefMut};

use ntex::http::Response;
use ntex::web::{Responder, WebResponseError};
use serde::Serialize;

use crate::error::AppErr;

const HEAD_RESP: &'static str = "resp";
const HEAD_ERR: &'static str = "err";
const HEAD_SUCC: &'static str = "succ";

const CONTENT_TYPE_BIN: &'static str = "application/octet-stream";

pub struct Cbor<T>(pub T);

impl<T> AsRef<T> for Cbor<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Cbor<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Cbor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Serialize> Responder for Cbor<T> {
    async fn respond_to(self, _req: &ntex::web::HttpRequest) -> Response {
        let body = serde_cbor::to_vec(&self.0).unwrap();
        Response::Ok()
            .set_header(HEAD_RESP, HEAD_SUCC)
            .content_type(CONTENT_TYPE_BIN)
            .body(body)
    }
}

pub type CborRes<T> = Result<Cbor<T>, AppErr>;

pub fn new_cbor<T>(val: T) -> CborRes<T> {
    Ok(Cbor(val))
}

#[derive(Debug, Serialize)]
struct ErrResp {
    err_code: i32,
    err_msg: String,
}

#[derive(Debug, Serialize)]
struct ErrResp2<'a> {
    err_code: i32,
    err_msg: &'a str,
}

impl WebResponseError for AppErr {
    fn error_response(&self, _: &ntex::web::HttpRequest) -> Response {
        let body = match self {
            Self::Custom(info) => serde_cbor::to_vec(info).unwrap(),

            _ => {
                let resp = ErrResp {
                    err_code: -1,
                    err_msg: self.to_string(),
                };
                serde_cbor::to_vec(&resp).unwrap()
            }
        };
        Response::Ok()
            .set_header(HEAD_RESP, HEAD_ERR)
            .content_type(CONTENT_TYPE_BIN)
            .body(body)
    }
}
