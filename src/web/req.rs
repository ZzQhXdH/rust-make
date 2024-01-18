use super::resp::Cbor;
use crate::error::{AppErr, ErrorExt, error};
use ntex::{web::{ErrorRenderer, FromRequest}, util::BytesMut};
use serde::de::DeserializeOwned;

const MAX_REQ_SIZE: usize = 50 * 1024 * 1024;

impl<T: DeserializeOwned + 'static, E: ErrorRenderer> FromRequest<E> for Cbor<T> {
    type Error = AppErr;

    async fn from_request(
        _req: &ntex::web::HttpRequest,
        payload: &mut ntex::http::Payload,
    ) -> Result<Self, Self::Error> {
        let mut buf = BytesMut::new();
        while let Some(item) = payload.recv().await {
            let chunk = item.wrap()?;
            if (buf.len() + chunk.len()) > MAX_REQ_SIZE {
                return error("请求数据体积过大");
            }
            buf.extend_from_slice(&chunk);
        }
        let req = serde_cbor::from_slice::<T>(&buf)?;
        Ok(Cbor(req))
    }
}
