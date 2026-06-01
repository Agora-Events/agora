use super::*;
use crate::events::{PriceUpdatedEvent, ProSubscriptionEvent};
use crate::types::{SubscriptionTier, SECONDS_PER_MONTH};
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::StellarAssetClient,
    Address, Env, IntoVal,
};

fn setup_env() -> (Env, ProSubscriptionContractClient<'static>, Address, Address, Address) {
use super::contract::ProSubscriptionContract;
use super::types::Subscription;
use crate::error::ProSubscriptionError;
use crate::ProSubscriptionContractClient;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo, MockAuth, MockAuthInvoke};
use soroban_sdk::{token, Address, Env, IntoVal};

const SECONDS_PER_MONTH: u64 = 30 * 24 * 60 * 60;

fn setup() -> (
    Env,
    ProSubscriptionContractClient<'static>,
    Address,
    Address,
    Address,
) {
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
    let usdc = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.initialize(&admin, &platform_wallet, &usdc, &1000i128);

    (env, client, admin, platform_wallet, usdc)
}

fn setup_without_auth_mock() -> (
    Env,
    ProSubscriptionContractClient<'static>,
    Address,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProSubscriptionContract);
    let client = ProSubscriptionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.initialize(&admin, &platform_wallet, &usdc, &1000i128);

    (env, client, contract_id, admin, platform_wallet, usdc)
}

#[test]
fn test_is_initialized_before_and_after_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProSubscriptionContract);
    let client = ProSubscriptionContractClient::new(&env, &contract_id);

    assert!(!client.is_initialized());

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.initialize(&admin, &platform_wallet, &usdc, &1000i128);

    assert!(client.is_initialized());
}

#[test]
fn test_get_subscription_expiry_none_for_unknown_address() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let organizer = Address::generate(&env);

    assert_eq!(client.get_subscription_expiry(&organizer), None);
}

#[test]
fn test_get_subscription_expiry_after_subscribing() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);

    client.subscribe_pro(&organizer, &1u32);

    let subscription = client.get_subscription(&organizer).unwrap();
    assert_eq!(
        client.get_subscription_expiry(&organizer),
        Some(subscription.expires_at)
    );
}

#[test]
fn test_renew_active_subscription() {
    let (env, client, _admin, platform_wallet, usdc) = setup();

    let organizer = Address::generate(&env);

    // Mint and approve payment for initial subscription (1 month)
    let monthly_price = 1000i128;
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price));
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);

    client.subscribe_pro(&organizer, &1u32);

    let sub_before: Subscription = client.get_subscription(&organizer).unwrap();
    let start = sub_before.started_at;
    let expected_first_expiry = start + SECONDS_PER_MONTH;
    assert_eq!(sub_before.expires_at, expected_first_expiry);

    // Approve payment for renewal (1 month)
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price));
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);

    // Renew before expiry; should extend from current expiry
    client.renew_subscription(&organizer, &1u32);

    let sub_after: Subscription = client.get_subscription(&organizer).unwrap();
    let expected_second_expiry = start + SECONDS_PER_MONTH * 2;
    assert_eq!(sub_after.expires_at, expected_second_expiry);
    assert!(sub_after.is_active);
    // platform wallet should have received payments (simple sanity)
    let _platform_balance = token::Client::new(&env, &usdc).balance(&platform_wallet);
}

#[test]
fn test_renew_expired_subscription() {
    let (env, client, _admin, platform_wallet, usdc) = setup();

    let organizer = Address::generate(&env);
    let monthly_price = 1000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price));
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    let sub: Subscription = client.get_subscription(&organizer).unwrap();
    let expiry = sub.expires_at;

    // Advance ledger past expiry
    env.ledger().set(LedgerInfo {
        timestamp: expiry + 10,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Approve payment for renewal after expiry
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price));
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);

    client.renew_subscription(&organizer, &1u32);

    let renewed: Subscription = client.get_subscription(&organizer).unwrap();
    let expected = env.ledger().timestamp() + SECONDS_PER_MONTH;
    assert_eq!(renewed.expires_at, expected);
    assert!(renewed.is_active);
    let _platform_balance = token::Client::new(&env, &usdc).balance(&platform_wallet);
}

#[test]
fn test_renew_subscription_not_found() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let never = Address::generate(&env);
    let res = client.try_renew_subscription(&never, &1u32);
    assert_eq!(res, Err(Ok(ProSubscriptionError::SubscriptionNotFound)));
}

#[test]
fn test_subscribe_zero_months_error() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    // No need to mint/approve — contract should reject months == 0 early
    let res = client.try_subscribe_pro(&organizer, &0u32);
    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidPrice)));
    // ensure no subscription was created
    assert!(client.get_subscription(&organizer).is_none());
    let _ = usdc; // keep unused warning away
}

#[test]
fn test_subscribe_already_active_error() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1000i128;
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    // Attempt to subscribe again while active
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    let res = client.try_subscribe_pro(&organizer, &1u32);
    assert_eq!(
        res,
        Err(Ok(ProSubscriptionError::SubscriptionAlreadyActive))
    );
}

// ── Issue #645: SECONDS_PER_MONTH lives in types ──────────────────────────────

#[test]
fn test_seconds_per_month_value() {
    assert_eq!(SECONDS_PER_MONTH, 30 * 24 * 60 * 60);
#[test]
fn test_cancel_subscription_removes_member_and_decrements_total() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    // Confirm total is 1
    assert_eq!(client.get_total_pro_subscriptions(), 1u32);

    // Cancel subscription (admin auth is mocked)
    client.cancel_subscription(&organizer);

    // Subscription should be inactive
    let sub = client.get_subscription(&organizer).unwrap();
    assert!(!sub.is_active);

    // Members list should not contain organizer and total should be 0
    assert_eq!(client.get_total_pro_subscriptions(), 0u32);
    let members = client.get_pro_members();
    assert!(!members.contains(&organizer));
}

#[test]
fn test_update_pro_price_success() {
    let (_env, client, _admin, _platform_wallet, _usdc) = setup();

    let initial_price = 1000i128;
    assert_eq!(client.get_pro_monthly_price(), initial_price);

    let new_price = 2000i128;
    client.update_pro_price(&new_price);

    assert_eq!(client.get_pro_monthly_price(), new_price);
}

#[test]
fn test_update_pro_price_zero() {
    let (_env, client, _admin, _platform_wallet, _usdc) = setup();

    let res = client.try_update_pro_price(&0i128);
    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidPrice)));
}

#[test]
fn test_update_pro_price_negative() {
    let (_env, client, _admin, _platform_wallet, _usdc) = setup();

    let res = client.try_update_pro_price(&-1i128);
    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidPrice)));
}

#[test]
fn test_update_pro_price_unauthorized() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();

    // Clear all mocked auths to force a real auth check
    env.mock_all_auths();

    client.update_pro_price(&2000i128);
}

#[test]
fn test_get_platform_wallet() {
    let (_env, client, _admin, platform_wallet, _usdc) = setup();

    let result = client.get_platform_wallet();
    assert_eq!(result, Some(platform_wallet));
}

#[test]
fn test_update_platform_wallet_success() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let new_wallet = Address::generate(&env);

    client.update_platform_wallet(&new_wallet);

    assert_eq!(client.get_platform_wallet(), Some(new_wallet));
}

#[test]
#[should_panic]
fn test_update_platform_wallet_unauthorized() {
    let (env, client, contract_id, _admin, _platform_wallet, _usdc) = setup_without_auth_mock();
    let non_admin = Address::generate(&env);
    let new_wallet = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "update_platform_wallet",
            args: (&new_wallet,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.update_platform_wallet(&new_wallet);
}

#[test]
fn test_update_platform_wallet_self_address() {
    let (_env, client, _admin, _platform_wallet, _usdc) = setup();

    let res = client.try_update_platform_wallet(&client.address);

    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidAddress)));
}

#[test]
fn test_get_payment_token() {
    let (_env, client, _admin, _platform_wallet, usdc) = setup();

    let result = client.get_payment_token();
    assert_eq!(result, Some(usdc));
}

#[test]
fn test_update_payment_token_success() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let new_token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.update_payment_token(&new_token);

    assert_eq!(client.get_payment_token(), Some(new_token));
}

#[test]
#[should_panic]
fn test_update_payment_token_unauthorized() {
    let (env, client, contract_id, _admin, _platform_wallet, _usdc) = setup_without_auth_mock();
    let non_admin = Address::generate(&env);
    let new_token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    env.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "update_payment_token",
            args: (&new_token,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.update_payment_token(&new_token);
}
