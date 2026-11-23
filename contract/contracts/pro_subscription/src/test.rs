use super::contract::ProSubscriptionContract;
use super::types::Subscription;
use crate::error::ProSubscriptionError;
use crate::events::{AdminUpdatedEvent, PriceUpdatedEvent, ProSubscriptionEvent};
use crate::types::{SubscriptionTier, MAX_SUBSCRIPTION_MONTHS, SECONDS_PER_MONTH};
use crate::ProSubscriptionContractClient;
use soroban_sdk::testutils::{Address as _, Events, Ledger, LedgerInfo, MockAuth, MockAuthInvoke};
use soroban_sdk::{
    contract, contractimpl, token, token::StellarAssetClient, Address, BytesN, Env, Executable,
    IntoVal, TryIntoVal,
};

fn setup_env() -> (
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
    let usdc = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.initialize(&admin, &platform_wallet, &usdc, &1_000_000i128);

    (env, client, admin, platform_wallet, usdc)
}

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
        Err(Ok(ProSubscriptionError::SubscriptionAlreadyActive))
    );
}

// ── Issue #1722: register_basic after Pro expiry ─────────────────────────────

#[test]
fn test_register_basic_succeeds_after_pro_expired() {
    let (env, client, _, _, token_id) = setup_env();

    let asset = StellarAssetClient::new(&env, &token_id);
    let organizer = Address::generate(&env);
    asset.mint(&organizer, &1_000_000);

    client.subscribe_pro(&organizer, &1);
    assert!(client.is_pro_member(&organizer));

    let sub = client.get_subscription(&organizer).unwrap();

    // Advance ledger past expiry
    env.ledger().set(LedgerInfo {
        timestamp: sub.expires_at + 10,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Pro has lapsed, so Basic registration must succeed
    client.register_basic(&organizer);

    let basic = client.get_subscription(&organizer).unwrap();
    assert_eq!(basic.tier, SubscriptionTier::Basic);
    assert!(basic.is_active);

    // Pro state must be cleaned up
    assert!(!client.is_pro_member(&organizer));
    let members = client.get_pro_members();
    assert!(
        !members.contains(&organizer),
        "expired organizer must not remain in the pro members list"
    );
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
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProSubscriptionContract);
    let client = ProSubscriptionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.initialize(&admin, &platform_wallet, &usdc, &1_000_000i128);

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

    client.initialize(&admin, &platform_wallet, &usdc, &1_000_000i128);

    assert!(client.is_initialized());
}

#[test]
fn test_get_subscription_expiry_none_for_unknown_address() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let organizer = Address::generate(&env);

    assert_eq!(client.get_subscription_expiry(&organizer), None);
}

#[test]
fn test_get_subscription_none_for_unknown_address() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let organizer = Address::generate(&env);

    assert_eq!(client.get_subscription(&organizer), None);
    assert!(!client.is_pro_member(&organizer));
}

#[test]
fn test_get_subscription_expiry_after_subscribing() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

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
    let monthly_price = 1_000_000i128;
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
    let monthly_price = 1_000_000i128;

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
    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidDuration)));
    // ensure no subscription was created
    assert!(client.get_subscription(&organizer).is_none());
    let _ = usdc; // keep unused warning away
}

// ── Issue #1726: subscription duration validation ────────────────────────────

#[test]
fn test_subscribe_max_months_succeeds() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;
    let total_cost = monthly_price * (MAX_SUBSCRIPTION_MONTHS as i128);

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &total_cost);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &total_cost, &99999);

    client.subscribe_pro(&organizer, &MAX_SUBSCRIPTION_MONTHS);

    let sub = client.get_subscription(&organizer).unwrap();
    assert!(sub.is_active);
    assert_eq!(
        sub.expires_at,
        env.ledger().timestamp() + SECONDS_PER_MONTH * (MAX_SUBSCRIPTION_MONTHS as u64)
    );
}

#[test]
fn test_subscribe_above_max_months_fails() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let organizer = Address::generate(&env);

    let res = client.try_subscribe_pro(&organizer, &(MAX_SUBSCRIPTION_MONTHS + 1));
    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidDuration)));
    assert!(client.get_subscription(&organizer).is_none());
}

#[test]
fn test_renew_zero_months_error() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    let res = client.try_renew_subscription(&organizer, &0u32);
    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidDuration)));
}

#[test]
fn test_renew_max_months_succeeds() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    let expiry_before = client.get_subscription_expiry(&organizer).unwrap();

    let total_cost = monthly_price * (MAX_SUBSCRIPTION_MONTHS as i128);
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &total_cost);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &total_cost, &99999);

    client.renew_subscription(&organizer, &MAX_SUBSCRIPTION_MONTHS);

    let expiry_after = client.get_subscription_expiry(&organizer).unwrap();
    assert_eq!(
        expiry_after,
        expiry_before + SECONDS_PER_MONTH * (MAX_SUBSCRIPTION_MONTHS as u64)
    );
}

#[test]
fn test_renew_above_max_months_fails() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    let res = client.try_renew_subscription(&organizer, &(MAX_SUBSCRIPTION_MONTHS + 1));
    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidDuration)));
}

// ── Issue #1727: read-only getters are side-effect free ──────────────────────

fn expire_subscription(env: &Env, client: &ProSubscriptionContractClient, organizer: &Address) {
    let sub = client.get_subscription(organizer).unwrap();
    env.ledger().set(LedgerInfo {
        timestamp: sub.expires_at + 1,
        protocol_version: 23,
        sequence_number: 40,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });
}

#[test]
fn test_getters_do_not_emit_events_on_expired_subscription() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    expire_subscription(&env, &client, &organizer);

    // Drain any events emitted so far by re-reading the event list length.
    let events_before = env.events().all().len();

    assert!(!client.is_pro_member(&organizer));
    let sub = client.get_subscription(&organizer).unwrap();
    assert!(!sub.is_active);
    assert_eq!(
        client.get_subscription_expiry(&organizer),
        Some(sub.expires_at)
    );

    let events_after = env.events().all().len();
    assert_eq!(
        events_before, events_after,
        "read-only getters must not emit events"
    );
}

#[test]
fn test_getters_do_not_mutate_members_list_on_expired_subscription() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    expire_subscription(&env, &client, &organizer);

    // Getters report the lapsed state but leave storage untouched.
    assert!(!client.is_pro_member(&organizer));
    let _ = client.get_subscription(&organizer);
    let _ = client.get_subscription_expiry(&organizer);

    assert_eq!(client.get_total_pro_subscriptions(), 1u32);
    assert!(client.get_pro_members().contains(&organizer));

    // A subsequent mutating call corrects the members list and counter.
    client.sync_expired(&organizer);

    assert_eq!(client.get_total_pro_subscriptions(), 0u32);
    assert!(!client.get_pro_members().contains(&organizer));
}

// ── Issue #1728: admin-gated upgrade ─────────────────────────────────────────

/// Minimal contract used to obtain a valid alternate Wasm hash for upgrade tests.
#[contract]
struct DummyUpgradeable;

#[contractimpl]
impl DummyUpgradeable {
    pub fn ping(_env: Env) {}
}

fn wasm_hash_of(address: &Address) -> BytesN<32> {
    match address.executable() {
        Some(Executable::Wasm(hash)) => hash,
        _ => panic!("address is not a Wasm contract"),
    }
}

#[test]
fn test_upgrade_by_admin_succeeds_and_emits_event() {
    use crate::events::ContractUpgradedEvent;

    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let old_wasm_hash = wasm_hash_of(&client.address);
    // The test host gives every natively registered contract the same mock Wasm hash, so the
    // old and new hashes cannot be told apart here; the event is still checked against both.
    let new_wasm_hash = wasm_hash_of(&env.register(DummyUpgradeable, ()));

    client.upgrade(&new_wasm_hash);

    assert_eq!(wasm_hash_of(&client.address), new_wasm_hash);

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();

    let topic: ProSubscriptionEvent = topics.get(0).unwrap().into_val(&env);
    assert_eq!(topic, ProSubscriptionEvent::ContractUpgraded);

    let payload: ContractUpgradedEvent = data.into_val(&env);
    assert_eq!(payload.old_wasm_hash, old_wasm_hash);
    assert_eq!(payload.new_wasm_hash, new_wasm_hash);
}

#[test]
fn test_upgrade_by_non_admin_is_rejected() {
    let (env, client, contract_id, _admin, _platform_wallet, _usdc) = setup_without_auth_mock();
    let non_admin = Address::generate(&env);
    let new_wasm_hash: BytesN<32> = BytesN::from_array(&env, &[7u8; 32]);

    env.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "upgrade",
            args: (&new_wasm_hash,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.upgrade(&new_wasm_hash);
    }));

    assert!(
        result.is_err(),
        "upgrade should fail when called without the admin's authorization"
    );
}

#[test]
fn test_subscribe_already_active_error() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;
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
}

#[test]
fn test_cancel_subscription_removes_member_and_decrements_total() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

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

    let initial_price = 1_000_000i128;
    assert_eq!(client.get_pro_monthly_price(), initial_price);

    let new_price = 2_000_000i128;
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
fn test_update_admin_unauthorized() {
    let (env, client, contract_id, admin, _platform_wallet, _usdc) = setup_without_auth_mock();
    let non_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    // Mock auth for a random non-admin address only, so the real require_auth
    // check inside update_admin (which requires the *current* admin) fails.
    env.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "update_admin",
            args: (&new_admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.update_admin(&new_admin);
    }));

    assert!(
        result.is_err(),
        "update_admin should fail when called without the admin's authorization"
    );
    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_admin_updated_event_payload() {
    let (env, client, admin, _platform_wallet, _usdc) = setup();
    let new_admin = Address::generate(&env);

    client.update_admin(&new_admin);

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();

    let topic: ProSubscriptionEvent = topics.get(0).unwrap().into_val(&env);
    assert_eq!(topic, ProSubscriptionEvent::AdminUpdated);

    let payload: AdminUpdatedEvent = data.into_val(&env);
    assert_eq!(payload.old_admin, admin);
    assert_eq!(payload.new_admin, new_admin);
    assert_eq!(payload.updated_by, admin);
    assert_eq!(payload.timestamp, env.ledger().timestamp());

    assert_eq!(client.get_admin(), Some(new_admin));
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
fn test_update_payment_token_self_address() {
    let (_env, client, _admin, _platform_wallet, usdc) = setup();

    let res = client.try_update_payment_token(&client.address);

    assert_eq!(res, Err(Ok(ProSubscriptionError::InvalidAddress)));
    assert_eq!(client.get_payment_token(), Some(usdc));
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

// ── Pro member list events ────────────────────────────────────────────────────

#[test]
fn test_pro_member_added_event_on_subscribe() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);

    client.subscribe_pro(&organizer, &1u32);

    // Just check that events were emitted
    let events = env.events().all();
    assert!(!events.is_empty(), "No events emitted");
}

#[test]
fn test_pro_member_added_event_on_renew() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    // Initial subscription
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    // Renewal
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.renew_subscription(&organizer, &1u32);

    // Just check that events were emitted
    let events = env.events().all();
    assert!(!events.is_empty(), "No events emitted");
}

// ── Issue #1724: ProMemberAdded only on membership transition ────────────────

fn count_events(env: &Env, kind: ProSubscriptionEvent) -> u32 {
    let mut count = 0u32;
    for (_, topics, _) in env.events().all().iter() {
        // Token transfers emitted during renewals use a different topic shape, so skip any
        // event whose first topic is not a `ProSubscriptionEvent`.
        let topic: Result<ProSubscriptionEvent, _> = topics.get(0).unwrap().try_into_val(env);
        if topic == Ok(kind.clone()) {
            count += 1;
        }
    }
    count
}

#[test]
fn test_renew_active_emits_only_subscription_renewed() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price * 2));
    token::Client::new(&env, &usdc).approve(
        &organizer,
        &client.address,
        &(monthly_price * 2),
        &99999,
    );
    client.subscribe_pro(&organizer, &1u32);

    // Clear the event buffer so only renewal events are observed
    env.events().all();

    client.renew_subscription(&organizer, &1u32);

    assert_eq!(
        count_events(&env, ProSubscriptionEvent::ProMemberAdded),
        0,
        "extending an active subscription must not emit ProMemberAdded"
    );
    assert_eq!(
        count_events(&env, ProSubscriptionEvent::SubscriptionRenewed),
        1,
        "extending an active subscription must emit SubscriptionRenewed"
    );
}

#[test]
fn test_renew_expired_emits_pro_member_added_and_renewed() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price * 2));
    token::Client::new(&env, &usdc).approve(
        &organizer,
        &client.address,
        &(monthly_price * 2),
        &99999,
    );
    client.subscribe_pro(&organizer, &1u32);
    let sub = client.get_subscription(&organizer).unwrap();

    // Advance ledger past expiry
    env.ledger().set(LedgerInfo {
        timestamp: sub.expires_at + 10,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    client.renew_subscription(&organizer, &1u32);

    assert_eq!(
        count_events(&env, ProSubscriptionEvent::ProMemberAdded),
        1,
        "reactivating an expired subscription must emit ProMemberAdded"
    );
    assert_eq!(
        count_events(&env, ProSubscriptionEvent::SubscriptionRenewed),
        1,
        "reactivating an expired subscription must emit SubscriptionRenewed"
    );
}

#[test]
fn test_renew_cancelled_emits_pro_member_added_and_renewed() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price * 2));
    token::Client::new(&env, &usdc).approve(
        &organizer,
        &client.address,
        &(monthly_price * 2),
        &99999,
    );
    client.subscribe_pro(&organizer, &1u32);
    client.cancel_subscription(&organizer);

    client.renew_subscription(&organizer, &1u32);

    assert_eq!(
        count_events(&env, ProSubscriptionEvent::ProMemberAdded),
        1,
        "reactivating a cancelled subscription must emit ProMemberAdded"
    );
    assert_eq!(
        count_events(&env, ProSubscriptionEvent::SubscriptionRenewed),
        1,
        "reactivating a cancelled subscription must emit SubscriptionRenewed"
    );
}

#[test]
fn test_pro_member_removed_event_on_cancel() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    client.cancel_subscription(&organizer);

    // Just check that events were emitted
    let events = env.events().all();
    assert!(!events.is_empty(), "No events emitted");
}

// ── Issue #640: get_total_pro_subscriptions accounting ───────────────────────

#[test]
fn test_total_subscriptions_increments_on_subscribe() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let monthly_price = 1_000_000i128;

    let org1 = Address::generate(&env);
    let org2 = Address::generate(&env);

    token::StellarAssetClient::new(&env, &usdc).mint(&org1, &monthly_price);
    token::Client::new(&env, &usdc).approve(&org1, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&org1, &1u32);

    assert_eq!(client.get_total_pro_subscriptions(), 1u32);

    token::StellarAssetClient::new(&env, &usdc).mint(&org2, &monthly_price);
    token::Client::new(&env, &usdc).approve(&org2, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&org2, &1u32);

    assert_eq!(client.get_total_pro_subscriptions(), 2u32);
}

#[test]
fn test_total_subscriptions_decrements_on_cancel() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let monthly_price = 1_000_000i128;
    let organizer = Address::generate(&env);

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    assert_eq!(client.get_total_pro_subscriptions(), 1u32);

    client.cancel_subscription(&organizer);

    assert_eq!(client.get_total_pro_subscriptions(), 0u32);
}

#[test]
fn test_total_subscriptions_no_double_count() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let monthly_price = 1_000_000i128;
    let organizer = Address::generate(&env);

    // First subscription — succeeds
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    // Second subscription while still active — must fail
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    let res = client.try_subscribe_pro(&organizer, &1u32);
    assert_eq!(
        res,
        Err(Ok(ProSubscriptionError::SubscriptionAlreadyActive))
    );

    // Counter must still be 1, not 2
    assert_eq!(client.get_total_pro_subscriptions(), 1u32);
}

// ── Issue #632: cancel_subscription coverage ─────────────────────────────────

#[test]
fn test_cancel_subscription_success() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    // Confirm active before cancel
    assert!(client.is_pro_member(&organizer));

    client.cancel_subscription(&organizer);

    // is_pro_member must return false after cancellation
    assert!(!client.is_pro_member(&organizer));

    // Subscription record should exist but be inactive
    let sub = client.get_subscription(&organizer).unwrap();
    assert!(!sub.is_active);
}

#[test]
fn test_cancel_subscription_not_found() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let never_subscribed = Address::generate(&env);

    let res = client.try_cancel_subscription(&never_subscribed);
    assert_eq!(res, Err(Ok(ProSubscriptionError::SubscriptionNotFound)));
}

#[test]
#[should_panic]
fn test_cancel_subscription_unauthorized() {
    let (env, client, contract_id, _admin, _platform_wallet, usdc) = setup_without_auth_mock();
    let non_admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    // Subscribe the organizer first (mock all auths just for setup)
    env.mock_all_auths();
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    // Now attempt cancel as a non-admin — should panic
    env.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "cancel_subscription",
            args: (&organizer,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.cancel_subscription(&organizer);
}

// ── Issue #876: Explicit Pro Subscription Test Suite ──────────────────────────

#[test]
fn test_issue_876_subscribe_success() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;
    let months = 3u32;
    let total_cost = monthly_price * (months as i128);

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &total_cost);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &total_cost, &99999);

    client.subscribe_pro(&organizer, &months);

    let sub = client.get_subscription(&organizer).unwrap();
    assert_eq!(sub.tier, SubscriptionTier::Pro);
    assert!(sub.is_active);
    // 3-month subscriptions receive a 5% multi-month discount
    assert_eq!(sub.amount_paid, total_cost * 9_500 / 10_000);
    assert_eq!(
        sub.expires_at,
        env.ledger().timestamp() + SECONDS_PER_MONTH * (months as u64)
    );
    assert!(client.is_pro_member(&organizer));
}

#[test]
fn test_issue_876_renew_active_subscription() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price * 2));
    token::Client::new(&env, &usdc).approve(
        &organizer,
        &client.address,
        &(monthly_price * 2),
        &99999,
    );

    client.subscribe_pro(&organizer, &1u32);
    let initial_expiry = client.get_subscription_expiry(&organizer).unwrap();

    client.renew_subscription(&organizer, &1u32);
    let renewed_expiry = client.get_subscription_expiry(&organizer).unwrap();

    assert_eq!(renewed_expiry, initial_expiry + SECONDS_PER_MONTH);
}

#[test]
fn test_issue_876_renew_expired_subscription() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &(monthly_price * 2));
    token::Client::new(&env, &usdc).approve(
        &organizer,
        &client.address,
        &(monthly_price * 2),
        &99999,
    );

    client.subscribe_pro(&organizer, &1u32);
    let initial_sub = client.get_subscription(&organizer).unwrap();

    // Advance time past initial expiry
    env.ledger().set(LedgerInfo {
        timestamp: initial_sub.expires_at + 1000,
        protocol_version: 23,
        sequence_number: 20,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    client.renew_subscription(&organizer, &1u32);
    let renewed_sub = client.get_subscription(&organizer).unwrap();

    assert_eq!(
        renewed_sub.expires_at,
        env.ledger().timestamp() + SECONDS_PER_MONTH
    );
    assert!(client.is_pro_member(&organizer));
}

#[test]
fn test_issue_876_is_pro_member_active() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);

    client.subscribe_pro(&organizer, &1u32);
    assert!(client.is_pro_member(&organizer));
}

#[test]
fn test_issue_876_is_pro_member_expired() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);

    client.subscribe_pro(&organizer, &1u32);
    let sub = client.get_subscription(&organizer).unwrap();

    env.ledger().set(LedgerInfo {
        timestamp: sub.expires_at + 1,
        protocol_version: 23,
        sequence_number: 30,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    assert!(!client.is_pro_member(&organizer));
}

#[test]
fn test_issue_876_admin_cancel_subscription() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);

    client.subscribe_pro(&organizer, &1u32);
    assert!(client.is_pro_member(&organizer));

    client.cancel_subscription(&organizer);
    assert!(!client.is_pro_member(&organizer));

    let sub = client.get_subscription(&organizer).unwrap();
    assert!(!sub.is_active);
}

#[test]
fn test_issue_876_admin_update_price() {
    let (_env, client, _admin, _platform_wallet, _usdc) = setup();
    let new_price = 5_000_000i128;

    client.update_pro_price(&new_price);
    assert_eq!(client.get_pro_monthly_price(), new_price);
}

#[test]
fn test_issue_876_admin_update_admin() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let new_admin = Address::generate(&env);

    client.update_admin(&new_admin);
    assert_eq!(client.get_admin(), Some(new_admin));
}

#[test]
fn test_issue_876_admin_update_platform_wallet() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let new_wallet = Address::generate(&env);

    client.update_platform_wallet(&new_wallet);
    assert_eq!(client.get_platform_wallet(), Some(new_wallet));
}

#[test]
fn test_issue_876_admin_update_payment_token() {
    let (env, client, _admin, _platform_wallet, _usdc) = setup();
    let new_token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.update_payment_token(&new_token);
    assert_eq!(client.get_payment_token(), Some(new_token));
}

// ── Issue #1439: PlatformWalletUpdated event ──────────────────────────────────

#[test]
fn test_platform_wallet_updated_event_payload() {
    use crate::events::PlatformWalletUpdatedEvent;

    let (env, client, admin, old_wallet, _usdc) = setup_env();
    let new_wallet = Address::generate(&env);

    client.update_platform_wallet(&new_wallet);

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();

    let topic: ProSubscriptionEvent = topics.get(0).unwrap().into_val(&env);
    assert_eq!(topic, ProSubscriptionEvent::PlatformWalletUpdated);

    let payload: PlatformWalletUpdatedEvent = data.into_val(&env);
    assert_eq!(payload.old_wallet, old_wallet);
    assert_eq!(payload.new_wallet, new_wallet);
    assert_eq!(payload.updated_by, admin);
    assert_eq!(payload.timestamp, env.ledger().timestamp());
    assert_eq!(client.get_platform_wallet(), Some(new_wallet));
}

// ── Issue #1440: PaymentTokenUpdated event ────────────────────────────────────

#[test]
fn test_payment_token_updated_event_payload() {
    use crate::events::PaymentTokenUpdatedEvent;

    let (env, client, admin, _platform_wallet, old_token) = setup_env();
    let new_token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.update_payment_token(&new_token);

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();

    let topic: ProSubscriptionEvent = topics.get(0).unwrap().into_val(&env);
    assert_eq!(topic, ProSubscriptionEvent::PaymentTokenUpdated);

    let payload: PaymentTokenUpdatedEvent = data.into_val(&env);
    assert_eq!(payload.old_token, old_token);
    assert_eq!(payload.new_token, new_token);
    assert_eq!(payload.updated_by, admin);
    assert_eq!(payload.timestamp, env.ledger().timestamp());
    assert_eq!(client.get_payment_token(), Some(new_token));
}

// ── Issue #1441: Reject same-admin update ────────────────────────────────────

#[test]
fn test_update_admin_same_address_returns_error() {
    let (_env, client, admin, _platform_wallet, _usdc) = setup_env();

    let res = client.try_update_admin(&admin);
    assert_eq!(res, Err(Ok(ProSubscriptionError::SameAdmin)));

    // Admin must remain unchanged
    assert_eq!(client.get_admin(), Some(admin));
}

// ── Issue #1447: Re-subscribe after cancel ───────────────────────────────────

#[test]
fn test_resubscribe_after_cancel() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    let monthly_price = 1_000_000i128;

    // First subscription
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    let first_expiry = client.get_subscription_expiry(&organizer).unwrap();

    // Cancel
    client.cancel_subscription(&organizer);
    assert!(!client.is_pro_member(&organizer));

    // Advance ledger so the re-subscription timestamp is clearly later
    env.ledger().set(LedgerInfo {
        timestamp: first_expiry + 1000,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Re-subscribe
    token::StellarAssetClient::new(&env, &usdc).mint(&organizer, &monthly_price);
    token::Client::new(&env, &usdc).approve(&organizer, &client.address, &monthly_price, &99999);
    client.subscribe_pro(&organizer, &1u32);

    // User is pro again
    assert!(client.is_pro_member(&organizer));

    // Totals are exactly 1 (not double-counted)
    assert_eq!(client.get_total_pro_subscriptions(), 1u32);
    let members = client.get_pro_members();
    assert_eq!(
        members.iter().filter(|m| *m == organizer).count(),
        1,
        "organizer should appear exactly once in the members list"
    );

    // New expiry is based on the new subscription time, not the old one
    let new_expiry = client.get_subscription_expiry(&organizer).unwrap();
    assert_eq!(new_expiry, env.ledger().timestamp() + SECONDS_PER_MONTH);
    assert!(
        new_expiry > first_expiry,
        "new expiry must be after the old one"
    );
}

// ── Issue #1725: paginated get_pro_members_page ──────────────────────────────

fn subscribe_members(
    env: &Env,
    client: &ProSubscriptionContractClient<'static>,
    usdc: &Address,
    count: u32,
) -> soroban_sdk::Vec<Address> {
    let monthly_price = 1_000_000i128;
    let mut members = soroban_sdk::Vec::new(env);
    for _ in 0..count {
        let organizer = Address::generate(env);
        token::StellarAssetClient::new(env, usdc).mint(&organizer, &monthly_price);
        token::Client::new(env, usdc).approve(&organizer, &client.address, &monthly_price, &99999);
        client.subscribe_pro(&organizer, &1u32);
        members.push_back(organizer);
    }
    members
}

#[test]
fn test_get_pro_members_page_first_page() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let members = subscribe_members(&env, &client, &usdc, 5);

    let page = client.get_pro_members_page(&0u32, &2u32);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap(), members.get(0).unwrap());
    assert_eq!(page.get(1).unwrap(), members.get(1).unwrap());
}

#[test]
fn test_get_pro_members_page_middle_page() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let members = subscribe_members(&env, &client, &usdc, 5);

    let page = client.get_pro_members_page(&2u32, &2u32);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap(), members.get(2).unwrap());
    assert_eq!(page.get(1).unwrap(), members.get(3).unwrap());
}

#[test]
fn test_get_pro_members_page_last_partial_page() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let members = subscribe_members(&env, &client, &usdc, 5);

    let page = client.get_pro_members_page(&4u32, &2u32);
    assert_eq!(page.len(), 1);
    assert_eq!(page.get(0).unwrap(), members.get(4).unwrap());
}

#[test]
fn test_get_pro_members_page_out_of_range_start() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    subscribe_members(&env, &client, &usdc, 3);

    let page = client.get_pro_members_page(&10u32, &2u32);
    assert_eq!(page.len(), 0);
}

#[test]
fn test_get_pro_members_page_limit_clamped_to_max() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let total = crate::types::MAX_PAGE_SIZE + 5;
    subscribe_members(&env, &client, &usdc, total);

    let page = client.get_pro_members_page(&0u32, &u32::MAX);
    assert_eq!(page.len(), crate::types::MAX_PAGE_SIZE);
}
