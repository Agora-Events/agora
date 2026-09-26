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

/// POST /upload/image
///
/// Accepts a `multipart/form-data` request with a single `file` field.
/// Validates file magic bytes (JPEG/PNG/WebP/GIF) and size (≤ 5 MB), then uploads to
/// S3/R2 under a UUID-based key and returns the public URL.
pub async fn upload_image(
    Extension(config): Extension<Config>,
    mut multipart: Multipart,
) -> Response {
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
}
