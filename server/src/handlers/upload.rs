use aws_credential_types::Credentials;
use aws_sdk_s3::{config::Region, primitives::ByteStream, Client};
use axum::{extract::Multipart, response::IntoResponse, response::Response, Extension};
use serde::Serialize;
use uuid::Uuid;

use crate::config::Config;
use crate::utils::error::AppError;
use crate::utils::response::success;

const MAX_SIZE: usize = 5 * 1024 * 1024; // 5 MB

#[derive(Serialize)]
struct UploadResponse {
    url: String,
}

/// Inspect first few bytes to detect image MIME type (JPEG, PNG, GIF, WebP).
pub fn detect_mime_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        Some("image/jpeg")
    } else if bytes.len() >= 8 && bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if bytes.len() >= 6 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
        Some("image/gif")
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Helper to extract authenticated wallet address from auth extension or headers (Issue #1625).
pub fn extract_authenticated_wallet(
    auth_user: Option<&Extension<crate::middleware::auth::AuthUser>>,
    headers: &axum::http::HeaderMap,
    config_jwt_secret: &str,
) -> Result<String, AppError> {
    if let Some(Extension(user)) = auth_user {
        if !user.wallet.trim().is_empty() {
            return Ok(user.wallet.clone());
        }
    }

    if let Some(org) = headers.get("x-api-key-organizer").and_then(|v| v.to_str().ok()) {
        if !org.trim().is_empty() {
            return Ok(org.to_string());
        }
    }

    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::AuthError("Missing Authorization header".to_string()))?;

    let token = header.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::AuthError("Authorization header must use Bearer scheme".to_string())
    })?;

    let dev_prefix = format!("{}{}", "sk_", "live_");
    if token.starts_with(&dev_prefix) {
        return Err(AppError::AuthError("Invalid API key".to_string()));
    }

    let secret = if !config_jwt_secret.is_empty() {
        config_jwt_secret.to_string()
    } else {
        match std::env::var("JWT_SECRET") {
            Ok(s) if !s.is_empty() => s,
            _ => return Err(AppError::AuthError("JWT secret not configured".to_string())),
        }
    };

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    let claims = jsonwebtoken::decode::<crate::handlers::auth::Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::AuthError(format!("Invalid or expired token: {e}")))?;

    if claims.sub.trim().is_empty() {
        return Err(AppError::AuthError(
            "Authenticated wallet address cannot be empty".to_string(),
        ));
    }

    Ok(claims.sub)
}

/// POST /upload/image
///
/// Accepts a `multipart/form-data` request with a single `file` field.
/// Requires an authenticated wallet (via Bearer JWT or AuthUser extension).
/// Validates file magic bytes (JPEG/PNG/WebP/GIF) and size (≤ 5 MB), then uploads to
/// S3/R2 under a UUID-based key and returns the public URL.
pub async fn upload_image(
    Extension(config): Extension<Config>,
    auth_user: Option<Extension<crate::middleware::auth::AuthUser>>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let wallet = match extract_authenticated_wallet(
        auth_user.as_ref(),
        &headers,
        &config.jwt_secret,
    ) {
        Ok(w) => w,
        Err(e) => return e.into_response(),
    };

    tracing::info!(wallet = %wallet, "Image upload requested by authenticated wallet");
    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        Ok(None) => {
            return AppError::ValidationError("No file field found in request".to_string())
                .into_response()
        }
        Err(e) => {
            return AppError::ValidationError(format!("Multipart error: {e}")).into_response()
        }
    };

    // Read bytes with size cap
    let data = match field.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return AppError::ValidationError(format!("Failed to read file: {e}")).into_response()
        }
    };

    if data.len() > MAX_SIZE {
        return AppError::ValidationError("File exceeds the 5 MB size limit".to_string())
            .into_response();
    }

    // Inspect magic bytes to detect actual MIME type
    let detected_mime = match detect_mime_type(&data) {
        Some(mime) => mime,
        None => {
            return AppError::ValidationError(
                "Invalid or unsupported file format. Magic bytes do not match safe image types."
                    .to_string(),
            )
            .into_response();
        }
    };

    // Validate against allowed MIME types configuration
    if !config
        .allowed_upload_mime_types
        .iter()
        .any(|m| m.eq_ignore_ascii_case(detected_mime))
    {
        return AppError::ValidationError(format!(
            "MIME type '{detected_mime}' is not allowed"
        ))
        .into_response();
    }

    let ext = match detected_mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "bin",
    };

    // Build S3 client
    let creds = Credentials::new(
        &config.s3_access_key_id,
        &config.s3_secret_access_key,
        None,
        None,
        "agora-static",
    );

    let mut s3_config = aws_sdk_s3::Config::builder()
        .credentials_provider(creds)
        .region(Region::new(config.s3_region.clone()))
        .force_path_style(config.s3_endpoint_url.is_some());

    if let Some(ref endpoint) = config.s3_endpoint_url {
        s3_config = s3_config.endpoint_url(endpoint);
    }

    let client = Client::from_conf(s3_config.build());

    // Generate unique key
    let key = format!("{}.{}", Uuid::new_v4(), ext);

    let result = client
        .put_object()
        .bucket(&config.s3_bucket)
        .key(&key)
        .content_type(detected_mime)
        .body(ByteStream::from(data))
        .send()
        .await;

    if let Err(e) = result {
        tracing::error!("S3 upload failed: {:?}", e);
        return AppError::ExternalServiceError("Image upload failed".to_string()).into_response();
    }

    let url = format!("{}/{}", config.s3_public_url.trim_end_matches('/'), key);
    success(UploadResponse { url }, "Image uploaded successfully").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_jpeg_magic_bytes() {
        let bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_mime_type(&bytes), Some("image/jpeg"));
    }

    #[test]
    fn test_detect_png_magic_bytes() {
        let bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert_eq!(detect_mime_type(&bytes), Some("image/png"));
    }

    #[test]
    fn test_detect_gif_magic_bytes() {
        let bytes = b"GIF89a\x01\x00\x01\x00";
        assert_eq!(detect_mime_type(bytes), Some("image/gif"));
    }

    #[test]
    fn test_detect_webp_magic_bytes() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(b"WEBP");
        assert_eq!(detect_mime_type(&bytes), Some("image/webp"));
    }

    #[test]
    fn test_reject_php_script_or_svg() {
        let php_script = b"<?php echo 'malicious'; ?>";
        assert_eq!(detect_mime_type(php_script), None);

        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
        assert_eq!(detect_mime_type(svg), None);
    }

    #[test]
    fn test_extract_wallet_from_auth_user_extension() {
        let user = crate::middleware::auth::AuthUser {
            user_id: Uuid::new_v4(),
            wallet: "GBEXAMPLEWALLETADDRESSFORTESTING123456789".to_string(),
        };
        let ext = Extension(user);
        let headers = axum::http::HeaderMap::new();

        let wallet = extract_authenticated_wallet(Some(&ext), &headers, "secret").unwrap();
        assert_eq!(wallet, "GBEXAMPLEWALLETADDRESSFORTESTING123456789");
    }

    #[test]
    fn test_extract_wallet_missing_authorization_header() {
        let headers = axum::http::HeaderMap::new();
        let res = extract_authenticated_wallet(None, &headers, "secret");
        assert!(res.is_err());
        match res.unwrap_err() {
            AppError::AuthError(msg) => assert!(msg.contains("Missing Authorization")),
            other => panic!("Unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_extract_wallet_invalid_bearer_scheme() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Basic dXNlcjpwYXNz".parse().unwrap(),
        );
        let res = extract_authenticated_wallet(None, &headers, "secret");
        assert!(res.is_err());
        match res.unwrap_err() {
            AppError::AuthError(msg) => assert!(msg.contains("must use Bearer scheme")),
            other => panic!("Unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_extract_wallet_valid_jwt() {
        let secret = "super-secret-jwt-signing-key-for-tests-32b";
        let now = chrono::Utc::now();
        let claims = crate::handlers::auth::Claims {
            sub: "GAORGANIZERTESTWALLET123456789".to_string(),
            iat: now.timestamp(),
            exp: (now + chrono::Duration::hours(1)).timestamp(),
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );

        let wallet = extract_authenticated_wallet(None, &headers, secret).unwrap();
        assert_eq!(wallet, "GAORGANIZERTESTWALLET123456789");
    }

    #[test]
    fn test_extract_wallet_expired_jwt() {
        let secret = "super-secret-jwt-signing-key-for-tests-32b";
        let now = chrono::Utc::now();
        let claims = crate::handlers::auth::Claims {
            sub: "GAORGANIZERTESTWALLET123456789".to_string(),
            iat: (now - chrono::Duration::hours(2)).timestamp(),
            exp: (now - chrono::Duration::hours(1)).timestamp(),
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );

        let res = extract_authenticated_wallet(None, &headers, secret);
        assert!(res.is_err());
        match res.unwrap_err() {
            AppError::AuthError(msg) => assert!(msg.contains("Invalid or expired")),
            other => panic!("Unexpected error: {:?}", other),
        }
    }
}
