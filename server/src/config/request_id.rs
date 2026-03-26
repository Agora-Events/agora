use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Generates a new UUID v4 as the request ID.
#[derive(Clone, Copy, Default)]
pub struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        let header_value = id.parse().ok()?;
        Some(RequestId::new(header_value))
    }
}

/// Layer that sets `x-request-id` on incoming requests (generates if absent).
pub fn set_request_id_layer() -> SetRequestIdLayer<UuidRequestId> {
    SetRequestIdLayer::new(
        REQUEST_ID_HEADER.parse().expect("valid header name"),
        UuidRequestId,
    )
}

/// Layer that propagates `x-request-id` from request to response.
pub fn propagate_request_id_layer() -> PropagateRequestIdLayer {
    PropagateRequestIdLayer::new(REQUEST_ID_HEADER.parse().expect("valid header name"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(propagate_request_id_layer())
            .layer(set_request_id_layer())
    }

    #[tokio::test]
    async fn test_request_id_header_is_set_on_response() {
        let router = test_router();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert!(
            resp.headers().contains_key(REQUEST_ID_HEADER),
            "response should contain x-request-id header"
        );
    }

    #[tokio::test]
    async fn test_request_id_is_valid_uuid() {
        let router = test_router();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = router.oneshot(req).await.unwrap();
        let id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .expect("x-request-id header should be present")
            .to_str()
            .expect("header value should be valid UTF-8");
        assert!(
            Uuid::parse_str(id).is_ok(),
            "x-request-id should be a valid UUID, got: {id}"
        );
    }

    #[tokio::test]
    async fn test_existing_request_id_is_propagated() {
        let router = test_router();
        let custom_id = Uuid::new_v4().to_string();
        let req = Request::builder()
            .uri("/")
            .header(REQUEST_ID_HEADER, &custom_id)
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        let returned_id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .expect("x-request-id header should be present")
            .to_str()
            .unwrap();
        assert_eq!(
            returned_id, custom_id,
            "pre-existing request ID should be preserved"
        );
    }

    #[tokio::test]
    async fn test_each_request_gets_unique_id() {
        let router = test_router();

        let id_a = {
            let req = Request::builder().uri("/").body(Body::empty()).unwrap();
            let resp = router.clone().oneshot(req).await.unwrap();
            resp.headers()
                .get(REQUEST_ID_HEADER)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        };

        let id_b = {
            let req = Request::builder().uri("/").body(Body::empty()).unwrap();
            let resp = router.oneshot(req).await.unwrap();
            resp.headers()
                .get(REQUEST_ID_HEADER)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        };

        assert_ne!(id_a, id_b, "each request should receive a unique ID");
    }
}
