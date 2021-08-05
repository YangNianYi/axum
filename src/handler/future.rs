//! Handler future types.

use crate::{
    body::{box_body, BoxBody},
    routing::RouteFuture,
};
use futures_util::ready;
use http::{
    header::{HeaderValue, CONTENT_LENGTH},
    Method, Request, Response,
};
use http_body::Empty;
use pin_project_lite::pin_project;
use std::convert::Infallible;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;

opaque_future! {
    /// The response future for [`IntoService`](super::IntoService).
    pub type IntoServiceFuture =
        futures_util::future::BoxFuture<'static, Result<Response<BoxBody>, Infallible>>;
}

pin_project! {
    /// Response future for [`OnMethod`](super::OnMethod).
    pub struct OnMethodResponseFuture<S, F, B>
    where
        S: Service<Request<B>>,
        F: Service<Request<B>>
    {
        #[pin]
        pub(super) f: RouteFuture<S, F, B>,
        pub(super) req_method: Method,
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

        if *this.req_method == Method::HEAD {
            res.headers_mut()
                .insert(CONTENT_LENGTH, HeaderValue::from_static("0"));
            res = res.map(|_| box_body(Empty::new()));
        }

        Poll::Ready(Ok(res))
    }
}
