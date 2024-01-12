use std::borrow::Cow;
use thiserror::Error;

pub type IoErr = std::io::Error;
pub type SqlxErr = sqlx::Error;

#[derive(Debug, Error)]
pub enum AppErr {

    #[error("io:{0}")]
    Io(#[from] IoErr),

    #[error("sqlx:{0}")]
    Sqlx(#[from] SqlxErr),

    #[error("wrap:{0}")]
    Wrap(Cow<'static, str>),

    #[error("custom:{0}")]
    Custom(i32, String),

    #[error("cbor:{0}")]
    Cbor(#[from] serde_cbor::Error),
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


impl <T, E: std::error::Error> ErrorExt<T> for Result<T, E> {

    fn wrap(self) -> Result<T, AppErr> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err( AppErr::Wrap(Cow::Owned(e.to_string())) )
        }
    }

    fn print_if_err(&self) {
        if let Err(e) = self {
            println!("err:{}", e);
        }
    }
}

impl <T> ErrorExt<T> for Option<T> {

    fn wrap(self) -> Result<T, AppErr> {
        match self {
            Some(v) => Ok(v),
            None => Err( AppErr::Wrap(Cow::Borrowed("none")) )
        }
    }

    fn print_if_err(&self) {
        if let None = self {
            println!("option none");
        }
    }
}


