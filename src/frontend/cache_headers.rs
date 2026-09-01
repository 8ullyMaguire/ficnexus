//! Static-asset cache headers (NEXT.md item 8).
//!
//! SvelteKit's adapter-static emits content-hashed assets under
//! `/_app/immutable/` (safe to cache forever) and HTML at the root (must
//! revalidate). This layer sets:
//!   - `/_app/immutable/*` → `public, max-age=31536000, immutable`
//!   - everything else (HTML, etc.) → `no-cache`
//! API routes are NOT routed here (this wraps only the static fallback).

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{header, Request, Response};
use tower::{Layer, Service};

const IMMUTABLE_MAX_AGE: u64 = 31536000; // 1 year

#[derive(Clone, Default)]
pub struct CacheHeadersLayer;

impl CacheHeadersLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for CacheHeadersLayer {
    type Service = CacheHeadersService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheHeadersService { inner: Arc::new(inner) }
    }
}

#[derive(Clone)]
pub struct CacheHeadersService<S> {
    inner: Arc<S>,
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CacheHeadersService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = Infallible;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut inner = (*self.inner).clone();
        inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = (*self.inner).clone();
        let path = req.uri().path().to_string();
        Box::pin(async move {
            let mut resp = inner.call(req).await?;
            let cache = if path.starts_with("/_app/immutable/") {
                format!("public, max-age={IMMUTABLE_MAX_AGE}, immutable")
            } else {
                "no-cache".to_string()
            };
            resp.headers_mut().insert(
                header::CACHE_CONTROL,
                cache.parse().unwrap_or_else(|_| header::HeaderValue::from_static("no-cache")),
            );
            Ok(resp)
        })
    }
}

// Keep `Body` imported for the trait signature used by callers/tests.
#[allow(unused_imports)]
use Body as _Body;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use std::convert::Infallible;
    use tower::ServiceExt;

    /// Echo service: returns a 200 empty response, letting the layer do the
    /// header insertion. No network, no DB — pure middleware wiring.
    async fn echo_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::new(req.into_body()))
    }

    #[tokio::test]
    async fn immutable_assets_get_long_cache() {
        let svc = CacheHeadersLayer::new().layer(tower::service_fn(echo_service));
        let resp = svc
            .oneshot(
                Request::builder()
                    .uri("/_app/immutable/entry/start.abc123.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let cc = resp.headers().get(header::CACHE_CONTROL).unwrap();
        assert_eq!(cc, "public, max-age=31536000, immutable");
    }

    #[tokio::test]
    async fn html_and_other_paths_get_no_cache() {
        let svc = CacheHeadersLayer::new().layer(tower::service_fn(echo_service));

        for path in ["/", "/fic/abc", "/manifest.webmanifest", "/_app/immutable-x"] {
            let resp = svc
                .clone()
                .oneshot(
                    Request::builder().uri(path).body(Body::empty()).unwrap(),
                )
                .await
                .unwrap();
            let cc = resp.headers().get(header::CACHE_CONTROL).unwrap();
            assert_eq!(cc, "no-cache", "path {path} must be no-cache");
        }
    }

    #[tokio::test]
    async fn nested_immutable_subpaths_are_cached() {
        let svc = CacheHeadersLayer::new().layer(tower::service_fn(echo_service));
        let resp = svc
            .oneshot(
                Request::builder()
                    .uri("/_app/immutable/chunks/foo.123.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=31536000, immutable"
        );
    }
}
