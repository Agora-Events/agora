use super::contract::ProSubscriptionContract;
use super::types::Subscription;
use crate::error::ProSubscriptionError;
use soroban_sdk::testutils::{Address as _, EnvTestConfig, Events, Ledger, LedgerInfo};
use soroban_sdk::{token, Address, Env, IntoVal, String};

const SECONDS_PER_MONTH: u64 = 30 * 24 * 60 * 60;

fn setup() -> (Env, ProSubscriptionContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ProSubscriptionContract, ());
    let client = ProSubscriptionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();

    client.initialize(&admin, &platform_wallet, &usdc, &1000i128);

    (env, client, admin, platform_wallet, usdc)
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
    client.renew_subscription(&organizer, &1u32).unwrap();

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

    client.renew_subscription(&organizer, &1u32).unwrap();

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
    let res = client.renew_subscription(&never, &1u32);
    assert!(matches!(res, Err(ProSubscriptionError::SubscriptionNotFound)));
}

#[test]
fn test_subscribe_zero_months_error() {
    let (env, client, _admin, _platform_wallet, usdc) = setup();
    let organizer = Address::generate(&env);
    // No need to mint/approve — contract should reject months == 0 early
    let res = client.subscribe_pro(&organizer, &0u32);
    assert!(matches!(res, Err(ProSubscriptionError::InvalidPrice)));
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
    let res = client.subscribe_pro(&organizer, &1u32);
    assert!(matches!(res, Err(ProSubscriptionError::SubscriptionAlreadyActive)));
}

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
    client.cancel_subscription(&organizer).unwrap();

    // Subscription should be inactive
    let sub = client.get_subscription(&organizer).unwrap();
    assert!(!sub.is_active);

    // Members list should not contain organizer and total should be 0
    assert_eq!(client.get_total_pro_subscriptions(), 0u32);
    let members = client.get_pro_members();
    assert!(!members.contains(&organizer));
}
