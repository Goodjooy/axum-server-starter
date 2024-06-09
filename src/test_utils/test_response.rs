use axum::BoxError;
use bytes::Bytes;
use futures::{future::ok, TryStreamExt};
use http::{response::Parts, Response};
use http_body::Body;
use http_body_util::combinators::UnsyncBoxBody;
use serde::de::DeserializeOwned;
use tap::Tap;

pub struct TestResponse {
    parts: Parts,
    payload: axum::body::Body,
}

impl TestResponse {
    pub fn new<R>(resp: Response<R>) -> Self
    where
        R: http_body::Body<Data = Bytes> + Send + 'static,
        R::Error: Into<BoxError>,
    {
        let (parts, body) = resp.into_parts();
        let payload = axum::body::Body::new(UnsyncBoxBody::new(body));

        Self { parts, payload }
    }

    pub fn parts(self) -> Parts {
        self.parts
    }

    pub async fn bytes(self) -> Result<Bytes, axum::Error> {
        let size = self.payload.size_hint().lower() as _;
        let ret = self
            .payload
            .into_data_stream()
            .try_fold(Vec::with_capacity(size), |buff, chunk| {
                ok(buff.tap_mut(|buff| buff.extend(chunk)))
            })
            .await
            .expect("Error");

        Ok(Bytes::from(ret))
    }

    pub async fn plain(self) -> Result<String, axum::Error> {
        let bytes = self.bytes().await?;
        String::from_utf8(bytes.to_vec()).map_err(axum::Error::new)
    }

    pub async fn json<T: DeserializeOwned>(self) -> Result<T, axum::Error> {
        let bytes = self.bytes().await?;
        let reader = std::io::Cursor::new(bytes);
        serde_json::from_reader(reader).map_err(axum::Error::new)
    }
}
