use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display};
use thiserror::Error;

pub type IoErr = std::io::Error;
pub type SqlxErr = sqlx::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrInfo {
    pub err_code: i32,
    pub err_msg: String,
}

impl Display for ErrInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "err_code:{}, err_msg:{}", self.err_code, self.err_msg)
    }
}

#[derive(Debug, Error)]
pub enum AppErr {
    #[error("io:{0}")]
    Io(#[from] IoErr),

    #[error("sqlx:{0}")]
    Sqlx(#[from] SqlxErr),

    #[error("wrap:{0}")]
    Wrap(Cow<'static, str>),

    #[error("custom:{0}")]
    Custom(ErrInfo),

    #[error("cbor:{0}")]
    Cbor(#[from] serde_cbor::Error),

    #[error("proto:{0}")]
    Proto(&'static str),
}

impl AppErr {

    pub fn into_info(self) -> ErrInfo {
        match self {
            Self::Custom(info) => info,
            _ => ErrInfo { err_code: -1, err_msg: self.to_string() }
        }
    }

    pub fn serial_to_vec(&self) -> Vec<u8> {
        match self {
            Self::Custom(info) => serde_cbor::to_vec(info).unwrap(),
            _ => {
                let info = ErrInfo {
                    err_code: -1,
                    err_msg: self.to_string(),
                };
                serde_cbor::to_vec(&info).unwrap()
            }
        }
    }
}

pub fn serial_to_vec<T: Serialize>(ret: Result<T, AppErr>) -> Vec<u8> {
    match ret {
        Ok(v) => serde_cbor::to_vec(&v).unwrap(),
        Err(e) => e.serial_to_vec(),
    }
}

pub fn proto_err<T>(msg: &'static str) -> Result<T, AppErr> {
    Err(AppErr::Proto(msg))
}

pub fn error<T>(msg: &'static str) -> Result<T, AppErr> {
    Err(AppErr::Wrap(Cow::Borrowed(msg)))
}

pub fn errors<T>(msg: String) -> Result<T, AppErr> {
    Err(AppErr::Wrap(Cow::Owned(msg)))
}

pub trait ErrorExt<T> {
    fn wrap(self) -> Result<T, AppErr>;

    fn print_if_err(&self);
}

impl<T, E: std::error::Error> ErrorExt<T> for Result<T, E> {
    fn wrap(self) -> Result<T, AppErr> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(AppErr::Wrap(Cow::Owned(e.to_string()))),
        }
    }

    fn print_if_err(&self) {
        if let Err(e) = self {
            println!("err:{}", e);
        }
    }
}

impl<T> ErrorExt<T> for Option<T> {
    fn wrap(self) -> Result<T, AppErr> {
        match self {
            Some(v) => Ok(v),
            None => Err(AppErr::Wrap(Cow::Borrowed("none"))),
        }
    }

    fn print_if_err(&self) {
        if let None = self {
            println!("option none");
        }
    }
}
