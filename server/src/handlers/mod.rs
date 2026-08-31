pub mod affiliates;
pub mod api_keys;
pub mod auth;
pub mod categories;
pub mod events;
pub mod follows;
pub mod geo;
pub mod governance;
pub mod health;
pub mod indexer;
pub mod leaderboard;
pub mod marketplace;
pub mod monitoring;
pub mod pricing;
pub mod profile;
pub mod qr_payload;
pub mod rates;
pub mod soroban_listener;
pub mod sync;
pub mod tickets;
pub mod waiting_room;
pub mod webhooks;
pub mod ws;
pub mod zk_checkin;

use axum::{extract::Path, response::IntoResponse, response::Response};

use crate::utils::error::{ApiError, AppError};
use crate::utils::response::empty_success;

/// Example endpoint demonstrating the standardised [`ApiError`] response shape.
pub async fn example_validation_error() -> Result<(), ApiError> {
    Err(ApiError::from(AppError::ValidationError(
        "The provided input is invalid".to_string(),
    )))
}

/// Example endpoint demonstrating a not-found [`ApiError`].
pub async fn example_not_found(Path(resource_id): Path<String>) -> Result<(), ApiError> {
    Err(ApiError::from(AppError::NotFound(format!(
        "Resource with id '{}' was not found",
        resource_id
    ))))
}

pub async fn example_empty_success() -> Response {
    empty_success("Operation completed successfully").into_response()
}

pub use crate::handlers::{
    auth::{logout, request_nonce, verify_signature},
    categories::{get_category, list_categories, CategoryState},
    events::{
        export_attendees_csv, flag_event, get_attendee_count, get_checkin_stats, get_event,
        get_event_counts, get_event_organizer, get_event_share_link, get_event_social_proof,
        get_events_map, get_ratings_summary, list_event_attendees, list_event_ratings,
        list_event_tickets, list_events, list_events_by_category, list_past_events,
        list_similar_events, list_ticket_tiers, list_upcoming_events, search_events,
        set_event_featured, submit_event_rating, toggle_event_flag, EventState,
    },
    health::{
        health_check, health_check_blockchain, health_check_db, health_check_ready,
        health_check_redis,
    },
    marketplace::{
        cancel_listing, create_key_envelope, create_listing, create_offer, get_key_envelope,
        get_listing, list_listings, list_offers, register_push_token, MarketplaceState,
    },
    profile::{
        delete_profile, get_my_profile, get_organizer_stats, get_profile_by_address,
        get_wallet_tickets, list_events_by_organizer, list_my_transactions, patch_profile,
        upsert_profile, ProfileState,
    },
    qr_payload::{
        delete_qr_payload, generate_attendee_qr, generate_qr_payload, list_event_qr_codes,
        list_qr_payloads, mark_qr_used, scan_ticket, verify_qr_payload,
    },
    rates::{get_rates, RatesState},
    soroban_listener::{spawn_listener, ListenerConfig},
    sync::{delta_sync, sync_status, SyncState},
    waiting_room::{
        join_queue, queue_status, queue_stream, request_challenge, WaitingRoomConfig,
        WaitingRoomState,
    },
    ws::{ws_purchases_handler, PurchaseBroadcaster},
};
