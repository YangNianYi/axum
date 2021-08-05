//! [`Service`](tower::Service) future types.

use crate::{
    body::{box_body, BoxBody},
    response::IntoResponse,
    routing::RouteFuture,
};
use bytes::Bytes;
use futures_util::ready;
use http::{
    header::{HeaderValue, CONTENT_LENGTH},
    Method, Request, Response,
};
use http_body::Empty;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{BoxError, Service};

pin_project! {
    /// Response future for [`HandleError`](super::HandleError).
    #[derive(Debug)]
    pub struct HandleErrorFuture<Fut, F> {
        #[pin]
        pub(super) inner: Fut,
        pub(super) f: Option<F>,
    }
}

impl<Fut, F, E, E2, B, Res> Future for HandleErrorFuture<Fut, F>
where
    Fut: Future<Output = Result<Response<B>, E>>,
    F: FnOnce(E) -> Result<Res, E2>,
    Res: IntoResponse,
    B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
    B::Error: Into<BoxError> + Send + Sync + 'static,
{
    type Output = Result<Response<BoxBody>, E2>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match ready!(this.inner.poll(cx)) {
            Ok(res) => Ok(res.map(box_body)).into(),
            Err(err) => {
                let f = this.f.take().unwrap();
                match f(err) {
                    Ok(res) => Ok(res.into_response().map(box_body)).into(),
                    Err(err) => Err(err).into(),
                }
            }
        }
    }
}

pin_project! {
    /// Response future for [`crate::handler::OnMethod`] and
    /// [`crate::service::OnMethod`].
    pub struct OnMethodResponseFuture<S, F, B>
    where
        S: Service<Request<B>>,
        F: Service<Request<B>>
    {
        #[pin]
        pub(crate) f: RouteFuture<S, F, B>,
        pub(crate) req_method: Method,
    }
}

impl<S, F, B> Future for OnMethodResponseFuture<S, F, B>
where
    S: Service<Request<B>, Response = Response<BoxBody>>,
    F: Service<Request<B>, Response = Response<BoxBody>, Error = S::Error>,
{
    type Output = Result<Response<BoxBody>, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut res: Response<_> = ready!(this.f.poll(cx)?);

        // HEAD must not contain a body according to
        // https://httpwg.org/specs/rfc7231.html#HEAD
        if *this.req_method == Method::HEAD {
            res.headers_mut()
                .insert(CONTENT_LENGTH, HeaderValue::from_static("0"));
            res = res.map(|_| box_body(Empty::new()));
        }

        Poll::Ready(Ok(res))
    }
}
