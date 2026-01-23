use axum::http::{Request, Response};
use std::{
    env,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// Security header names
const X_CONTENT_TYPE_OPTIONS: &str = "X-Content-Type-Options";
const X_FRAME_OPTIONS: &str = "X-Frame-Options";
const X_XSS_PROTECTION: &str = "X-XSS-Protection";
const STRICT_TRANSPORT_SECURITY: &str = "Strict-Transport-Security";
const CONTENT_SECURITY_POLICY: &str = "Content-Security-Policy";
const REFERRER_POLICY: &str = "Referrer-Policy";
const PERMISSIONS_POLICY: &str = "Permissions-Policy";

/// Security header values
const NOSNIFF: &str = "nosniff";
const DENY: &str = "DENY";
const XSS_BLOCK: &str = "1; mode=block";
const HSTS_VALUE: &str = "max-age=31536000; includeSubDomains";
const CSP_API_VALUE: &str = "default-src 'none'; frame-ancestors 'none'";
const REFERRER_POLICY_VALUE: &str = "strict-origin-when-cross-origin";
const PERMISSIONS_POLICY_VALUE: &str = "geolocation=(), microphone=(), camera=()";
#[derive(Clone)]
pub struct SecurityHeadersLayer {
    include_hsts: bool,
}

impl SecurityHeadersLayer {
    pub fn new(include_hsts: bool) -> Self {
        Self { include_hsts }
    }

    pub fn from_env() -> Self {
        let is_production = env::var("RUST_ENV")
            .map(|v| v.to_lowercase() == "production")
            .unwrap_or(false);

        if is_production {
            tracing::info!("Security: HSTS header enabled (production mode)");
        } else {
            tracing::info!("Security: HSTS header disabled (development mode)");
        }

        Self::new(is_production)
    }
}

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersService {
            inner,
            include_hsts: self.include_hsts,
        }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersService<S> {
    inner: S,
    include_hsts: bool,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SecurityHeadersService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = SecurityHeadersFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        SecurityHeadersFuture {
            future: self.inner.call(request),
            include_hsts: self.include_hsts,
        }
    }
}

#[pin_project::pin_project]
pub struct SecurityHeadersFuture<F> {
    #[pin]
    future: F,
    include_hsts: bool,
}

impl<F, ResBody, E> std::future::Future for SecurityHeadersFuture<F>
where
    F: std::future::Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.future.poll(cx) {
            Poll::Ready(Ok(mut response)) => {
                let headers = response.headers_mut();

                // Add security headers
                headers.insert(X_CONTENT_TYPE_OPTIONS, NOSNIFF.parse().unwrap());
                headers.insert(X_FRAME_OPTIONS, DENY.parse().unwrap());
                headers.insert(X_XSS_PROTECTION, XSS_BLOCK.parse().unwrap());
                headers.insert(CONTENT_SECURITY_POLICY, CSP_API_VALUE.parse().unwrap());
                headers.insert(REFERRER_POLICY, REFERRER_POLICY_VALUE.parse().unwrap());
                headers.insert(
                    PERMISSIONS_POLICY,
                    PERMISSIONS_POLICY_VALUE.parse().unwrap(),
                );

                // Only add HSTS in production (HTTPS environments)
                if *this.include_hsts {
                    headers.insert(STRICT_TRANSPORT_SECURITY, HSTS_VALUE.parse().unwrap());
                }

                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub fn create_security_headers_layer() -> SecurityHeadersLayer {
    SecurityHeadersLayer::from_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_headers_layer_creation() {
        let layer = SecurityHeadersLayer::new(false);
        assert!(!layer.include_hsts);

        let layer_with_hsts = SecurityHeadersLayer::new(true);
        assert!(layer_with_hsts.include_hsts);
    }

    #[test]
    fn test_from_env_defaults_to_no_hsts() {
        // Without RUST_ENV set to production, HSTS should be disabled
        std::env::remove_var("RUST_ENV");
        let layer = SecurityHeadersLayer::from_env();
        assert!(!layer.include_hsts);
    }
}
