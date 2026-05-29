use super::*;
use crate::events::{PriceUpdatedEvent, ProSubscriptionEvent};
use crate::types::{SubscriptionTier, SECONDS_PER_MONTH};
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::StellarAssetClient,
    Address, Env, IntoVal,
};

fn setup_env() -> (Env, ProSubscriptionContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProSubscriptionContract);
    let client = ProSubscriptionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &platform_wallet, &token_id, &1_000_000);
    (env, client, admin, platform_wallet, token_id)
}

// ── Issue #644: PriceUpdated event payload ────────────────────────────────────

#[test]
fn test_price_updated_event_payload() {
    let (env, client, admin, _, _) = setup_env();

    let old_price = client.get_pro_monthly_price();
    let new_price = 2_000_000_i128;
    client.update_pro_price(&new_price);

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();

    let topic: ProSubscriptionEvent = topics.get(0).unwrap().into_val(&env);
    assert_eq!(topic, ProSubscriptionEvent::PriceUpdated);

    let payload: PriceUpdatedEvent = data.into_val(&env);
    assert_eq!(payload.old_price, old_price);
    assert_eq!(payload.new_price, new_price);
    assert_eq!(payload.updated_by, admin);
}

// ── Issue #646: get_pro_members_count ─────────────────────────────────────────

#[test]
fn test_get_pro_members_count_zero_initially() {
    let (_, client, _, _, _) = setup_env();
    assert_eq!(client.get_pro_members_count(), 0);
}

#[test]
fn test_get_pro_members_count_after_subscriptions() {
    let (env, client, _, _, token_id) = setup_env();

    let asset = StellarAssetClient::new(&env, &token_id);
    let org1 = Address::generate(&env);
    let org2 = Address::generate(&env);
    asset.mint(&org1, &1_000_000);
    asset.mint(&org2, &1_000_000);

    client.subscribe_pro(&org1, &1);
    client.subscribe_pro(&org2, &1);

    assert_eq!(client.get_pro_members_count(), 2);
}

// ── Issue #647: register_basic ────────────────────────────────────────────────

#[test]
fn test_register_basic_happy_path() {
    let (env, client, _, _, _) = setup_env();
    let organizer = Address::generate(&env);

    client.register_basic(&organizer);

    let sub = client.get_subscription(&organizer).unwrap();
    assert_eq!(sub.tier, SubscriptionTier::Basic);
    assert_eq!(sub.expires_at, 0);
    assert!(sub.is_active);
    assert_eq!(sub.amount_paid, 0);
}

#[test]
fn test_register_basic_blocked_when_already_pro() {
    let (env, client, _, _, token_id) = setup_env();

    let asset = StellarAssetClient::new(&env, &token_id);
    let organizer = Address::generate(&env);
    asset.mint(&organizer, &1_000_000);

    client.subscribe_pro(&organizer, &1);

    let result = client.try_register_basic(&organizer);
    assert_eq!(
        result,
        Err(Ok(ProSubscriptionError::SubscriptionAlreadyActive))
    );
}

// ── Issue #645: SECONDS_PER_MONTH lives in types ──────────────────────────────

#[test]
fn test_seconds_per_month_value() {
    assert_eq!(SECONDS_PER_MONTH, 30 * 24 * 60 * 60);
}
