use crate::utils::error::AppError;
use axum::{extract::Request, middleware::Next, response::Response};
use axum_extra::extract::cookie::CookieJar;

pub async fn check_csrf(req: Request, next: Next) -> Result<Response, AppError> {
    let jar = CookieJar::from_headers(req.headers());
    let method = req.method();
    if method == axum::http::Method::GET
        || method == axum::http::Method::HEAD
        || method == axum::http::Method::OPTIONS
    {
        return Ok(next.run(req).await);
    }

    let path = req.uri().path();
    if path.contains("/auth/nonce") || path.contains("/auth/verify") {
        return Ok(next.run(req).await);
    }

    let cookie_token = jar.get("XSRF-TOKEN").map(|c| c.value().to_string());
    let header_token = req
        .headers()
        .get("X-XSRF-TOKEN")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if let (Some(cookie), Some(header)) = (cookie_token, header_token) {
        if cookie == header && !cookie.is_empty() {
            return Ok(next.run(req).await);
        }
    }

    Err(AppError::AuthError("CSRF validation failed".to_string()))
}
