use super::resp::Cbor;
use crate::error::{errors, AppErr, ErrorExt};
use ntex::web::{ErrorRenderer, FromRequest, WebResponseError};
use serde::de::DeserializeOwned;

const MAX_REQ_SIZE: usize = 50 * 1024 * 1024;

impl<T: DeserializeOwned + 'static, E: ErrorRenderer> FromRequest<E> for Cbor<T> {
    type Error = AppErr;

    async fn from_request(
        _req: &ntex::web::HttpRequest,
        payload: &mut ntex::http::Payload,
    ) -> Result<Self, Self::Error> {
        let buf = payload.recv().await.wrap()?.wrap()?;
        if buf.len() > MAX_REQ_SIZE {
            return errors(format!("请求数据过大"));
        }
        let req = serde_cbor::from_slice::<T>(&buf)?;
        Ok(Cbor(req))
    }
}
