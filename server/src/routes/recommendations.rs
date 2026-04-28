// server/src/routes/recommendations.rs
//
// Mount this inside your versioned router in main.rs / lib.rs:
//
//   use crate::routes::recommendations::recommendations_router;
//   let app = Router::new()
//       .nest("/api/v1", recommendations_router())
//       ...

use axum::{middleware, routing::get, Router};
use sqlx::PgPool;

use crate::{
    handlers::recommendations::get_recommended_events,
    middleware::auth::require_auth,
};

pub fn recommendations_router(pool: PgPool) -> Router {
    Router::new()
        .route(
            "/recommendations/events",
            get(get_recommended_events)
                // Requires a valid JWT — AuthUser is injected by this layer
                .layer(middleware::from_fn(require_auth)),
        )
        .with_state(pool)
}