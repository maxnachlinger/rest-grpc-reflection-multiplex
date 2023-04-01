use std::{
    error::Error,
    pin::Pin,
    task::{Context, Poll},
};

use http::HeaderMap;
use http_body::Body;

type EitherError = Box<dyn Error + Send + Sync + 'static>;

pub enum EitherBody<A, B> {
    A(A),
    B(B),
}

impl<A, B> Body for EitherBody<A, B>
where
    A: Body + Send + Unpin,
    B: Body<Data = A::Data> + Send + Unpin,
    A::Error: Into<EitherError>,
    B::Error: Into<EitherError>,
{
    type Data = A::Data;
    type Error = EitherError;

    fn is_end_stream(&self) -> bool {
        match self {
            EitherBody::A(body) => body.is_end_stream(),
            EitherBody::B(body) => body.is_end_stream(),
        }
    }

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.get_mut() {
            EitherBody::A(body) => Pin::new(body)
                .poll_data(cx)
                .map(|err| err.map(|e| e.map_err(Into::into))),
            EitherBody::B(body) => Pin::new(body)
                .poll_data(cx)
                .map(|err| err.map(|e| e.map_err(Into::into))),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        match self.get_mut() {
            EitherBody::A(body) => Pin::new(body).poll_trailers(cx).map_err(Into::into),
            EitherBody::B(body) => Pin::new(body).poll_trailers(cx).map_err(Into::into),
        }
    }
}
