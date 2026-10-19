//! Tests for the max_per_user ticket limit functionality.
//!
//! This test file verifies that:
//! 1. Users cannot exceed the per-user limit for a tier
//! 2. The limit is enforced across multiple purchases
//! 3. Different users have independent limits
//! 4. Refunds properly decrement the user's ticket count
//! 5. A limit of 0 means unlimited (no restriction)

use event_registry::{
    testutils::{event_registry::EventRegistryClient, register_event::register_event},
    types::{EventRegistrationArgs, TicketTier},
    storage,
    Address, Env, Map, String,
};

const VALID_CID: &str = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";

fn create_test_tier(env: &Env, name: &str, price: i128, limit: i128, max_per_user: u32) -> TicketTier {
    TicketTier {
        name: String::from_str(env, name),
        price,
        tier_limit: limit,
        current_sold: 0,
        is_refundable: true,
        auction_config: soroban_sdk::vec![env],
        loyalty_multiplier: 1,
        max_per_user,
    }
}

fn setup_test_event(env: &Env, client: &EventRegistryClient, organizer: &Address, max_per_user: u32) -> String {
    let mut tiers = Map::new(env);
    tiers.set(
        String::from_str(env, "vip"),
        create_test_tier(env, "VIP", 1000, 10, max_per_user),
    );
    tiers.set(
        String::from_str(env, "general"),
        create_test_tier(env, "General", 500, 20, max_per_user),
    );

    let event_id = String::from_str(env, "test_event");
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: Address::generate(env),
        metadata_cid: String::from_str(env, VALID_CID),
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
    });
    event_id
}

#[test]
fn test_per_user_limit_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(event_registry::EventRegistry, ());
    let client = event_registry::testutils::EventRegistryClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);
    
    let event_id = setup_test_event(&env, &client, &organizer, 3); // 3 tickets max per user
    let vip_tier = String::from_str(&env, "vip");
    let user1 = Address::generate(&env);
    
    // First 3 purchases should succeed
    client.increment_inventory(&event_id, &vip_tier, &user1, 1);
    client.increment_inventory(&event_id, &vip_tier, &user1, 1);
    client.increment_inventory(&event_id, &vip_tier, &user1, 1);
    
    // 4th purchase should fail
    let result = client.try_increment_inventory(&event_id, &vip_tier, &user1, 1);
    assert_eq!(result, Err(Ok(event_registry::error::EventRegistryError::PerUserLimitExceeded)));
    
    // Verify user has exactly 3 tickets
    let user_count = storage::get_user_ticket_count(&env, &event_id, &vip_tier, &user1);
    assert_eq!(user_count, 3);
}

#[test]
fn test_different_users_independent_limits() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(event_registry::EventRegistry, ());
    let client = event_registry::testutils::EventRegistryClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);
    
    let event_id = setup_test_event(&env, &client, &organizer, 2); // 2 tickets max per user
    let vip_tier = String::from_str(&env, "vip");
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    
    // User1 purchases 2 tickets (their limit)
    client.increment_inventory(&event_id, &vip_tier, &user1, 2);
    
    // User2 should also be able to purchase 2 tickets (independent limit)
    client.increment_inventory(&event_id, &vip_tier, &user2, 2);
    
    // Both users should now be at their limits
    let result1 = client.try_increment_inventory(&event_id, &vip_tier, &user1, 1);
    let result2 = client.try_increment_inventory(&event_id, &vip_tier, &user2, 1);
    
    assert_eq!(result1, Err(Ok(event_registry::error::EventRegistryError::PerUserLimitExceeded)));
    assert_eq!(result2, Err(Ok(event_registry::error::EventRegistryError::PerUserLimitExceeded)));
}

#[test]
fn test_unlimited_per_user_limit() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(event_registry::EventRegistry, ());
    let client = event_registry::testutils::EventRegistryClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);
    
    let event_id = setup_test_event(&env, &client, &organizer, 0); // 0 = unlimited
    let vip_tier = String::from_str(&env, "vip");
    let user1 = Address::generate(&env);
    
    // User should be able to purchase up to the tier limit (10)
    for _ in 0..10 {
        client.increment_inventory(&event_id, &vip_tier, &user1, 1);
    }
    
    // But not beyond the tier limit
    let result = client.try_increment_inventory(&event_id, &vip_tier, &user1, 1);
    assert_eq!(result, Err(Ok(event_registry::error::EventRegistryError::TierSoldOut)));
}

#[test]
fn test_refund_decrements_user_count() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(event_registry::EventRegistry, ());
    let client = event_registry::testutils::EventRegistryClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);
    
    let event_id = setup_test_event(&env, &client, &organizer, 5); // 5 tickets max per user
    let vip_tier = String::from_str(&env, "vip");
    let user1 = Address::generate(&env);
    
    // Purchase 3 tickets
    client.increment_inventory(&event_id, &vip_tier, &user1, 3);
    assert_eq!(storage::get_user_ticket_count(&env, &event_id, &vip_tier, &user1), 3);
    
    // Refund 1 ticket
    client.decrement_inventory(&event_id, &vip_tier, &user1);
    assert_eq!(storage::get_user_ticket_count(&env, &event_id, &vip_tier, &user1), 2);
    
    // Should be able to purchase 3 more tickets (up to the limit again)
    client.increment_inventory(&event_id, &vip_tier, &user1, 3);
    assert_eq!(storage::get_user_ticket_count(&env, &event_id, &vip_tier, &user1), 5);
    
    // But not exceed the limit
    let result = client.try_increment_inventory(&event_id, &vip_tier, &user1, 1);
    assert_eq!(result, Err(Ok(event_registry::error::EventRegistryError::PerUserLimitExceeded)));
}

#[test]
fn test_multiple_tiers_independent_limits() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(event_registry::EventRegistry, ());
    let client = event_registry::testutils::EventRegistryClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);
    
    let event_id = setup_test_event(&env, &client, &organizer, 2); // 2 tickets max per user
    let vip_tier = String::from_str(&env, "vip");
    let general_tier = String::from_str(&env, "general");
    let user1 = Address::generate(&env);
    
    // User should be able to purchase 2 tickets from each tier (independent limits)
    client.increment_inventory(&event_id, &vip_tier, &user1, 2);
    client.increment_inventory(&event_id, &general_tier, &user1, 2);
    
    // Should be blocked from purchasing more from either tier
    let result_vip = client.try_increment_inventory(&event_id, &vip_tier, &user1, 1);
    let result_general = client.try_increment_inventory(&event_id, &general_tier, &user1, 1);
    
    assert_eq!(result_vip, Err(Ok(event_registry::error::EventRegistryError::PerUserLimitExceeded)));
    assert_eq!(result_general, Err(Ok(event_registry::error::EventRegistryError::PerUserLimitExceeded)));
    
    // But refund from one tier should not affect the other
    client.decrement_inventory(&event_id, &vip_tier, &user1);
    
    // Should be able to purchase from VIP tier again, but still blocked from general
    client.increment_inventory(&event_id, &vip_tier, &user1, 1);
    let result_general_after_refund = client.try_increment_inventory(&event_id, &general_tier, &user1, 1);
    
    assert_eq!(result_general_after_refund, Err(Ok(event_registry::error::EventRegistryError::PerUserLimitExceeded)));
}
