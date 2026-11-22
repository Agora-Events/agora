//! Tests for the `max_per_user` ticket limit (#1741).
//!
//! Ported from the old, never-compiled `test_max_per_user.rs` that sat at the
//! crate root, and rewritten against the current API. Covered here:
//!
//! 1. A user cannot exceed the per-user limit for a tier, across several purchases.
//! 2. Different users have independent limits.
//! 3. A limit of 0 means unlimited per user; only the tier limit applies.
//! 4. Per-user limits are tracked separately for each tier.
//!
//! The old file also described "refunds decrement the user's ticket count". The
//! contract does not do that today: `decrement_inventory` takes no user and
//! `storage::subtract_from_user_ticket_count` is never called, so that scenario
//! is deliberately not asserted here.

use super::*;
use crate::error::EventRegistryError;
use crate::types::{EventRegistrationArgs, TicketTier};
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String};

fn test_payment_address(env: &Env) -> Address {
    Address::from_string(&String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJXFF",
    ))
}

fn setup(env: &Env) -> EventRegistryClient<'static> {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&Address::generate(env));
    client
}

fn tier(env: &Env, name: &str, price: i128, tier_limit: i128, max_per_user: u32) -> TicketTier {
    TicketTier {
        name: String::from_str(env, name),
        price,
        tier_limit,
        current_sold: 0,
        is_refundable: true,
        auction_config: soroban_sdk::vec![&env],
        loyalty_multiplier: 1,
        max_per_user,
    }
}

/// Registers `evt_per_user` with a "vip" tier (limit 10) and a "general" tier
/// (limit 20), both using the given `max_per_user`.
fn register_event_with_tiers(
    env: &Env,
    client: &EventRegistryClient<'static>,
    max_per_user: u32,
) -> String {
    let mut tiers = Map::new(env);
    tiers.set(
        String::from_str(env, "vip"),
        tier(env, "VIP", 1000, 10, max_per_user),
    );
    tiers.set(
        String::from_str(env, "general"),
        tier(env, "General", 500, 20, max_per_user),
    );

    let event_id = String::from_str(env, "evt_per_user");
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(env, "Test Event"),
        organizer_address: Address::generate(env),
        payment_address: test_payment_address(env),
        metadata_cid: String::from_str(
            env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 30,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        start_time: 0,
        is_private: false,
        end_time: 0,
        transfer_lock_duration: 0,
        accepted_tokens: soroban_sdk::Vec::new(env),
        use_global_whitelist: true,
        category_ids: None,
        referral_rate_bps: None,
    });
    event_id
}

fn buy(
    env: &Env,
    client: &EventRegistryClient<'static>,
    event_id: &String,
    tier_id: &str,
    user: &Address,
    quantity: u32,
) {
    client.increment_inventory(
        event_id,
        &String::from_str(env, tier_id),
        user,
        &quantity,
        &Address::generate(env),
    );
}

/// Returns the contract error from a purchase, or `None` if it succeeded.
fn buy_error(
    env: &Env,
    client: &EventRegistryClient<'static>,
    event_id: &String,
    tier_id: &str,
    user: &Address,
    quantity: u32,
) -> Option<EventRegistryError> {
    match client.try_increment_inventory(
        event_id,
        &String::from_str(env, tier_id),
        user,
        &quantity,
        &Address::generate(env),
    ) {
        Err(Ok(error)) => Some(error),
        _ => None,
    }
}

fn user_ticket_count(
    env: &Env,
    client: &EventRegistryClient<'static>,
    event_id: &String,
    tier_id: &str,
    user: &Address,
) -> u32 {
    env.as_contract(&client.address, || {
        crate::storage::get_user_ticket_count(env, event_id, &String::from_str(env, tier_id), user)
    })
}

#[test]
fn test_per_user_limit_enforced_across_multiple_purchases() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let event_id = register_event_with_tiers(&env, &client, 3);
    let user = Address::generate(&env);

    // Three single-ticket purchases reach the limit of 3.
    buy(&env, &client, &event_id, "vip", &user, 1);
    buy(&env, &client, &event_id, "vip", &user, 1);
    buy(&env, &client, &event_id, "vip", &user, 1);

    // The fourth is rejected, and the user is left with exactly 3.
    let result = buy_error(&env, &client, &event_id, "vip", &user, 1);
    assert_eq!(result, Some(EventRegistryError::PerUserLimitExceeded));
    assert_eq!(user_ticket_count(&env, &client, &event_id, "vip", &user), 3);
}

#[test]
fn test_per_user_limit_rejects_a_single_purchase_that_would_exceed_it() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let event_id = register_event_with_tiers(&env, &client, 3);
    let user = Address::generate(&env);

    buy(&env, &client, &event_id, "vip", &user, 2);

    // 2 already held + 2 requested = 4 > 3, even though neither call alone exceeds it.
    let result = buy_error(&env, &client, &event_id, "vip", &user, 2);
    assert_eq!(result, Some(EventRegistryError::PerUserLimitExceeded));
    assert_eq!(user_ticket_count(&env, &client, &event_id, "vip", &user), 2);

    // The remaining allowance of 1 is still usable.
    buy(&env, &client, &event_id, "vip", &user, 1);
    assert_eq!(user_ticket_count(&env, &client, &event_id, "vip", &user), 3);
}

#[test]
fn test_different_users_have_independent_limits() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let event_id = register_event_with_tiers(&env, &client, 2);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // User 1 uses their whole allowance; user 2 still has theirs.
    buy(&env, &client, &event_id, "vip", &user1, 2);
    buy(&env, &client, &event_id, "vip", &user2, 2);

    assert_eq!(
        buy_error(&env, &client, &event_id, "vip", &user1, 1),
        Some(EventRegistryError::PerUserLimitExceeded)
    );
    assert_eq!(
        buy_error(&env, &client, &event_id, "vip", &user2, 1),
        Some(EventRegistryError::PerUserLimitExceeded)
    );
    assert_eq!(user_ticket_count(&env, &client, &event_id, "vip", &user1), 2);
    assert_eq!(user_ticket_count(&env, &client, &event_id, "vip", &user2), 2);
}

#[test]
fn test_zero_per_user_limit_is_only_capped_by_the_tier_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let event_id = register_event_with_tiers(&env, &client, 0); // 0 = unlimited per user
    let user = Address::generate(&env);

    // One user can buy the whole VIP tier (limit 10)...
    for _ in 0..10 {
        buy(&env, &client, &event_id, "vip", &user, 1);
    }

    // ...but not one more: the tier limit applies, not the per-user limit.
    let result = buy_error(&env, &client, &event_id, "vip", &user, 1);
    assert_eq!(result, Some(EventRegistryError::TierSoldOut));
}

#[test]
fn test_per_user_limits_are_tracked_per_tier() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let event_id = register_event_with_tiers(&env, &client, 2);
    let user = Address::generate(&env);

    // A full allowance in each tier is allowed independently.
    buy(&env, &client, &event_id, "vip", &user, 2);
    buy(&env, &client, &event_id, "general", &user, 2);

    assert_eq!(
        buy_error(&env, &client, &event_id, "vip", &user, 1),
        Some(EventRegistryError::PerUserLimitExceeded)
    );
    assert_eq!(
        buy_error(&env, &client, &event_id, "general", &user, 1),
        Some(EventRegistryError::PerUserLimitExceeded)
    );
    assert_eq!(user_ticket_count(&env, &client, &event_id, "vip", &user), 2);
    assert_eq!(user_ticket_count(&env, &client, &event_id, "general", &user), 2);
}
