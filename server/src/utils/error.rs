use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use tracing::{error, warn};

/// Reusable API error body returned by every error response.
///
/// Serializes as `{ "code": <u16>, "message": "..." }` so clients can reliably
/// read `error.code` / top-level `code` and display a user-friendly message.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ApiError {
    /// HTTP status code mirrored in the JSON body for client-side branching.
    pub code: u16,
    /// Human-readable error message safe to show to end users.
    pub message: String,
}

impl ApiError {
    /// Build an [`ApiError`] from an HTTP status and message.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code: status.as_u16(),
            message: message.into(),
        }
    }

    /// Convenience constructor for unexpected internal failures / panics.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, axum::Json(self)).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        Self {
            code: err.status_code().as_u16(),
            message: err.public_message(),
        }
    }
}

/// Classification of sqlx database errors for HTTP mapping and logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseErrorCategory {
    /// Connection or pool failures — alert-worthy infrastructure issues.
    Connection,
    /// Unique constraint violation.
    UniqueViolation,
    /// Foreign key constraint violation.
    ForeignKeyViolation,
    /// Other query-level failures.
    Query,
}

impl DatabaseErrorCategory {
    fn from_sqlx(err: &sqlx::Error) -> Self {
        match err {
            sqlx::Error::Io(_) | sqlx::Error::PoolClosed | sqlx::Error::PoolTimedOut => {
                Self::Connection
            }
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    Self::UniqueViolation
                } else if db_err.is_foreign_key_violation() {
                    Self::ForeignKeyViolation
                } else {
                    Self::Query
                }
            }
            _ => Self::Query,
        }
    }
}

/// Standardised application error enum.
///
/// Every variant maps to a well-defined HTTP status code and a machine-readable
/// error `code` string so that API consumers can react programmatically to
/// errors.
#[derive(Debug, Error)]
pub enum AppError {
    /// 400 – the caller supplied data that failed validation.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// 401 – the request is missing or carries an invalid auth token.
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// 403 – the caller is authenticated but not authorised for this action.
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// 404 – the requested resource does not exist.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// 409 – the request conflicts with the current state of the resource.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Database failure — status code depends on the underlying sqlx error kind.
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    /// 503 – a downstream service or database call is unreachable.
    #[error("External service error: {0}")]
    ExternalServiceError(String),

    /// 500 – catch-all for internal failures.
    #[error("Internal server error")]
    InternalServerError(String),
}

impl AppError {
    /// Return the HTTP status code that corresponds to this error variant.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::AuthError(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::DatabaseError(err) => match DatabaseErrorCategory::from_sqlx(err) {
                DatabaseErrorCategory::Connection => StatusCode::SERVICE_UNAVAILABLE,
                DatabaseErrorCategory::UniqueViolation
                | DatabaseErrorCategory::ForeignKeyViolation => StatusCode::CONFLICT,
                DatabaseErrorCategory::Query => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::ExternalServiceError(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Return a stable, machine-readable error code for the variant.
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::AuthError(_) => "AUTH_ERROR",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::DatabaseError(err) => match DatabaseErrorCategory::from_sqlx(err) {
                DatabaseErrorCategory::Connection => "DATABASE_UNAVAILABLE",
                DatabaseErrorCategory::UniqueViolation => "UNIQUE_VIOLATION",
                DatabaseErrorCategory::ForeignKeyViolation => "FOREIGN_KEY_VIOLATION",
                DatabaseErrorCategory::Query => "DATABASE_ERROR",
            },
            AppError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
        }
    }

    /// Return the public-facing message that is safe to expose in the API
    /// response.  Internal details (e.g. raw SQL errors) are intentionally
    /// omitted.
    pub fn public_message(&self) -> String {
        match self {
            AppError::ValidationError(msg)
            | AppError::AuthError(msg)
            | AppError::Forbidden(msg)
            | AppError::NotFound(msg)
            | AppError::Conflict(msg)
            | AppError::ExternalServiceError(msg)
            | AppError::InternalServerError(msg) => msg.clone(),
            AppError::DatabaseError(err) => match DatabaseErrorCategory::from_sqlx(err) {
                DatabaseErrorCategory::Connection => {
                    "Database service is temporarily unavailable".to_string()
                }
                DatabaseErrorCategory::UniqueViolation => {
                    "A resource with this identifier already exists".to_string()
                }
                DatabaseErrorCategory::ForeignKeyViolation => {
                    "The referenced resource does not exist".to_string()
                }
                DatabaseErrorCategory::Query => "A database error occurred".to_string(),
            },
        }
    }
}

/// Convert `AppError` into an Axum [`Response`].
///
/// The response body is a standardised JSON object:
///
/// ```json
/// {
///   "code": 404,
///   "message": "Resource with id '42' was not found"
/// }
/// ```
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let machine_code = self.error_code();
        let message = self.public_message();

        // Log *before* the message is moved into the JSON body.
        match &self {
            AppError::DatabaseError(e) => match DatabaseErrorCategory::from_sqlx(e) {
                DatabaseErrorCategory::Connection => {
                    error!(error = ?e, "Database connection error");
                }
                DatabaseErrorCategory::UniqueViolation
                | DatabaseErrorCategory::ForeignKeyViolation
                | DatabaseErrorCategory::Query => {
                    warn!(error = ?e, "Database query error");
                }
            },
            _ => {
                error!(
                    error = ?self,
                    code = machine_code,
                    http_status = status.as_u16(),
                    message,
                    "Application error"
                );
            }
        }

        ApiError::new(status, message).into_response()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;
    use sqlx::error::DatabaseError;

    // Helper: consume a Response and deserialise its JSON body.
    async fn body_json(resp: Response) -> serde_json::Value {
        let body = resp.into_body();
        let bytes = to_bytes(body, usize::MAX).await.expect("body read failed");
        serde_json::from_slice(&bytes).expect("invalid JSON body")
    }

    // -----------------------------------------------------------------------
    // status_code
    // -----------------------------------------------------------------------

    #[test]
    fn test_status_codes() {
        assert_eq!(
            AppError::ValidationError("x".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::AuthError("x".into()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Forbidden("x".into()).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::NotFound("x".into()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::ExternalServiceError("x".into()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            AppError::InternalServerError("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // -----------------------------------------------------------------------
    // error_code
    // -----------------------------------------------------------------------

    #[test]
    fn test_error_codes() {
        assert_eq!(
            AppError::ValidationError("x".into()).error_code(),
            "VALIDATION_ERROR"
        );
        assert_eq!(AppError::AuthError("x".into()).error_code(), "AUTH_ERROR");
        assert_eq!(AppError::Forbidden("x".into()).error_code(), "FORBIDDEN");
        assert_eq!(AppError::NotFound("x".into()).error_code(), "NOT_FOUND");
        assert_eq!(
            AppError::ExternalServiceError("x".into()).error_code(),
            "EXTERNAL_SERVICE_ERROR"
        );
        assert_eq!(
            AppError::InternalServerError("x".into()).error_code(),
            "INTERNAL_SERVER_ERROR"
        );
    }

    // -----------------------------------------------------------------------
    // public_message
    // -----------------------------------------------------------------------

    #[test]
    fn test_public_message_passthrough() {
        let msg = "email is required";
        assert_eq!(AppError::ValidationError(msg.into()).public_message(), msg);
    }

    #[test]
    fn test_public_message_auth() {
        assert_eq!(
            AppError::AuthError("token expired".into()).public_message(),
            "token expired"
        );
    }

    #[test]
    fn test_public_message_not_found() {
        assert_eq!(
            AppError::NotFound("post 99".into()).public_message(),
            "post 99"
        );
    }

    #[test]
    fn test_public_message_database_hides_details() {
        let raw_sql_error = sqlx::Error::RowNotFound;
        let err = AppError::DatabaseError(raw_sql_error);
        assert_eq!(err.public_message(), "A database error occurred");
    }

    #[test]
    fn test_public_message_connection_error_is_generic() {
        let err = AppError::DatabaseError(sqlx::Error::PoolTimedOut);
        assert_eq!(
            err.public_message(),
            "Database service is temporarily unavailable"
        );
    }

    #[test]
    fn test_public_message_unique_violation_is_generic() {
        let err = AppError::DatabaseError(mock_db_error(MockDbErrorKind::UniqueViolation));
        assert_eq!(
            err.public_message(),
            "A resource with this identifier already exists"
        );
        assert!(!err.public_message().contains("duplicate"));
    }

    #[test]
    fn test_public_message_foreign_key_violation_is_generic() {
        let err = AppError::DatabaseError(mock_db_error(MockDbErrorKind::ForeignKeyViolation));
        assert_eq!(
            err.public_message(),
            "The referenced resource does not exist"
        );
        assert!(!err.public_message().contains("foreign key"));
    }

    #[test]
    fn test_public_message_external_service() {
        assert_eq!(
            AppError::ExternalServiceError("timeout".into()).public_message(),
            "timeout"
        );
    }

    #[test]
    fn test_public_message_internal() {
        assert_eq!(
            AppError::InternalServerError("unexpected panic".into()).public_message(),
            "unexpected panic"
        );
    }

    // -----------------------------------------------------------------------
    // IntoResponse — HTTP status codes
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_into_response_validation_error_status() {
        let resp = AppError::ValidationError("bad input".into()).into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_into_response_auth_error_status() {
        let resp = AppError::AuthError("invalid token".into()).into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_into_response_forbidden_status() {
        let resp = AppError::Forbidden("access denied".into()).into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_into_response_not_found_status() {
        let resp = AppError::NotFound("item 7".into()).into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_into_response_external_service_status() {
        let resp = AppError::ExternalServiceError("upstream timeout".into()).into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_into_response_internal_server_error_status() {
        let resp = AppError::InternalServerError("oops".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_into_response_database_error_status() {
        let resp = AppError::DatabaseError(sqlx::Error::RowNotFound).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_into_response_pool_timeout_returns_503() {
        let resp = AppError::DatabaseError(sqlx::Error::PoolTimedOut).into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let json = body_json(resp).await;
        assert_eq!(json["code"], 503);
        assert_eq!(
            json["message"],
            "Database service is temporarily unavailable"
        );
        assert!(!json["message"].as_str().unwrap().contains("timeout"));
    }

    #[tokio::test]
    async fn test_into_response_pool_closed_returns_503() {
        let resp = AppError::DatabaseError(sqlx::Error::PoolClosed).into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_into_response_unique_violation_returns_409() {
        let resp = AppError::DatabaseError(mock_db_error(MockDbErrorKind::UniqueViolation))
            .into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let json = body_json(resp).await;
        assert_eq!(json["code"], 409);
        assert_eq!(
            json["message"],
            "A resource with this identifier already exists"
        );
    }

    #[tokio::test]
    async fn test_into_response_foreign_key_violation_returns_409() {
        let resp = AppError::DatabaseError(mock_db_error(MockDbErrorKind::ForeignKeyViolation))
            .into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let json = body_json(resp).await;
        assert_eq!(json["code"], 409);
        assert_eq!(json["message"], "The referenced resource does not exist");
    }

    // -----------------------------------------------------------------------
    // IntoResponse — JSON body shape
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_into_response_body_has_flat_code_and_message() {
        let resp = AppError::ValidationError("oops".into()).into_response();
        let json = body_json(resp).await;
        assert_eq!(json["code"], 400);
        assert_eq!(json["message"], "oops");
        assert!(json.get("success").is_none());
        assert!(json.get("error").is_none());
    }

    #[tokio::test]
    async fn test_api_error_into_response_shape() {
        let resp = ApiError::new(StatusCode::NOT_FOUND, "thing").into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let json = body_json(resp).await;
        assert_eq!(json["code"], 404);
        assert_eq!(json["message"], "thing");
    }

    #[tokio::test]
    async fn test_into_response_validation_error_body() {
        let resp = AppError::ValidationError("name is required".into()).into_response();
        let json = body_json(resp).await;
        assert_eq!(json["code"], 400);
        assert_eq!(json["message"], "name is required");
    }

    #[tokio::test]
    async fn test_into_response_auth_error_body() {
        let resp = AppError::AuthError("token missing".into()).into_response();
        let json = body_json(resp).await;
        assert_eq!(json["code"], 401);
        assert_eq!(json["message"], "token missing");
    }

    #[tokio::test]
    async fn test_into_response_forbidden_body() {
        let resp = AppError::Forbidden("not allowed".into()).into_response();
        let json = body_json(resp).await;
        assert_eq!(json["code"], 403);
        assert_eq!(json["message"], "not allowed");
    }

    #[tokio::test]
    async fn test_into_response_not_found_body() {
        let resp = AppError::NotFound("record 42".into()).into_response();
        let json = body_json(resp).await;
        assert_eq!(json["code"], 404);
        assert_eq!(json["message"], "record 42");
    }

    #[tokio::test]
    async fn test_into_response_external_service_body() {
        let resp =
            AppError::ExternalServiceError("payment gateway unreachable".into()).into_response();
        let json = body_json(resp).await;
        assert_eq!(json["code"], 503);
        assert_eq!(json["message"], "payment gateway unreachable");
    }

    #[tokio::test]
    async fn test_into_response_internal_server_error_body() {
        let resp = AppError::InternalServerError("crash".into()).into_response();
        let json = body_json(resp).await;
        assert_eq!(json["code"], 500);
        assert_eq!(json["message"], "crash");
    }

    #[tokio::test]
    async fn test_into_response_database_error_hides_details() {
        let resp = AppError::DatabaseError(sqlx::Error::RowNotFound).into_response();
        let json = body_json(resp).await;
        assert_eq!(json["code"], 500);
        assert_eq!(json["message"], "A database error occurred");
    }

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    #[derive(Debug, Copy, Clone)]
    enum MockDbErrorKind {
        UniqueViolation,
        ForeignKeyViolation,
    }

    #[derive(Debug)]
    struct MockDbError {
        kind: MockDbErrorKind,
    }

    impl std::fmt::Display for MockDbError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "mock database error")
        }
    }

    impl std::error::Error for MockDbError {}

    impl DatabaseError for MockDbError {
        fn message(&self) -> &str {
            "duplicate key value violates unique constraint users_email_key"
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }

        fn kind(&self) -> sqlx::error::ErrorKind {
            match self.kind {
                MockDbErrorKind::UniqueViolation => sqlx::error::ErrorKind::UniqueViolation,
                MockDbErrorKind::ForeignKeyViolation => sqlx::error::ErrorKind::ForeignKeyViolation,
            }
        }
    }

    fn mock_db_error(kind: MockDbErrorKind) -> sqlx::Error {
        sqlx::Error::Database(Box::new(MockDbError { kind }))
    }

    // -----------------------------------------------------------------------
    // Content-Type header
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_into_response_content_type_is_json() {
        use axum::http::header::CONTENT_TYPE;
        let resp = AppError::NotFound("x".into()).into_response();
        let ct = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("application/json"),
            "Expected application/json content type, got: {ct}"
        );
    }
}
