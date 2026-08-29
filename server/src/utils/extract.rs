//! # Request extraction utilities
//!
//! Provides [`ValidatedJson`], a drop-in replacement for `axum::Json` that maps
//! *deserialisation failures* (including `serde(deny_unknown_fields)` rejections)
//! to a `400 Bad Request` [`AppError`] whose message names the offending field.
//!
//! By default Axum answers malformed/mis-typed JSON bodies with `422
//! Unprocessable Entity`. For our public request DTOs we prefer a plain `400`
//! so clients get a consistent, machine-readable validation error (see
//! `utils::error::AppError`).

use axum::async_trait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;

use crate::utils::error::AppError;

/// Wrapper around `axum::Json` that converts extraction failures into a `400`
/// [`AppError::ValidationError`] instead of the default `422`.
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(ValidatedJson(value.0)),
            Err(rejection) => Err(AppError::ValidationError(json_rejection_message(&rejection))),
        }
    }
}

/// Turn an Axum JSON rejection into a human-readable message that names the
/// offending field (serde's `deny_unknown_fields` error already includes the
/// unknown field name, e.g. `unknown field \`organiser_name\``).
fn json_rejection_message(rejection: &JsonRejection) -> String {
    match rejection {
        JsonRejection::JsonDataError(err) => err.body_text(),
        JsonRejection::JsonSyntaxError(err) => err.body_text(),
        JsonRejection::MissingJsonContentType(_) => {
            "Request must include a `Content-Type: application/json` header".to_string()
        }
        JsonRejection::BytesRejection(err) => err.body_text(),
        other => other.body_text(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header::CONTENT_TYPE, Request, StatusCode};

    #[derive(Debug, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Sample {
        name: String,
    }

    async fn extract_sample(body: &str) -> Result<Sample, AppError> {
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        match ValidatedJson::<Sample>::from_request(req, &()).await {
            Ok(ValidatedJson(v)) => Ok(v),
            Err(e) => Err(e),
        }
    }

    #[tokio::test]
    async fn extra_field_returns_400_naming_field() {
        let err = extract_sample(r#"{"name":"x","organiser_name":"y"}"#)
            .await
            .unwrap_err();
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert!(
            err.public_message().contains("organiser_name"),
            "expected message to name the unknown field, got: {}",
            err.public_message()
        );
    }

    #[tokio::test]
    async fn valid_payload_succeeds() {
        let v = extract_sample(r#"{"name":"x"}"#).await.unwrap();
        assert_eq!(v.name, "x");
    }

    #[tokio::test]
    async fn syntax_error_returns_400() {
        let err = extract_sample(r#"{"name":}"#).await.unwrap_err();
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }
}
