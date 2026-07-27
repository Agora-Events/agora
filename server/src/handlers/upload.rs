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

/// POST /upload/image
///
/// Accepts a `multipart/form-data` request with a single `file` field.
/// Validates content-type (JPEG/PNG) and size (≤ 5 MB), then uploads to
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

    // Validate content-type
    let content_type = field.content_type().unwrap_or("").to_string();

    let ext = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        _ => {
            return AppError::ValidationError("Only JPEG and PNG images are accepted".to_string())
                .into_response()
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
        .content_type(&content_type)
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
