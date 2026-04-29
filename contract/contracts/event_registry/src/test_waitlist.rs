use super::*;
use crate::error::EventRegistryError;
use crate::types::EventRegistrationArgs;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, Map, String,
};

fn test_payment_address(env: &Env) -> Address {
    Address::from_string(&String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJXFF",
    ))
}

/// Helper function to setup contract and initialize it
fn setup_contract(env: &Env) -> (EventRegistryClient, Address, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    
    (client, admin, platform_wallet)
}

/// Helper function to register a test event
fn register_test_event(
    env: &Env,
    client: &EventRegistryClient,
    event_id: &String,
    organizer: &Address,
) {
    let metadata_cid = String::from_str(
        env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(env);
    
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(env),
        metadata_cid,
        max_supply: 100,
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
    });
}

#[test]
fn test_join_waitlist_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _platform_wallet) = setup_contract(&env);
    let organizer = Address::generate(&env);
    let user = Address::generate(&env);
    let event_id = String::from_str(&env, "event_waitlist_test");
    
    // Register an event
    register_test_event(&env, &client, &event_id, &organizer);
    
    // Drain setup events
    let _ = env.events().all();
    
    // Join waitlist
    client.join_waitlist(&event_id, &user);
    
    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "expected WaitlistJoined event");
}

#[test]
fn test_join_waitlist_duplicate_fails() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _platform_wallet) = setup_contract(&env);
    let organizer = Address::generate(&env);
    let user = Address::generate(&env);
    let event_id = String::from_str(&env, "event_duplicate_test");
    
    // Register an event
    register_test_event(&env, &client, &event_id, &organizer);
    
    // Join waitlist first time - should succeed
    client.join_waitlist(&event_id, &user);
    
    // Try to join again - should fail
    let result = client.try_join_waitlist(&event_id, &user);
    assert_eq!(result, Err(Ok(EventRegistryError::AlreadyOnWaitlist)));
}

#[test]
fn test_join_waitlist_nonexistent_event_fails() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _platform_wallet) = setup_contract(&env);
    let user = Address::generate(&env);
    let fake_event_id = String::from_str(&env, "nonexistent_event");
    
    // Try to join waitlist for non-existent event
    let result = client.try_join_waitlist(&fake_event_id, &user);
    assert_eq!(result, Err(Ok(EventRegistryError::EventNotFound)));
}

#[test]
fn test_join_waitlist_no_payment_required() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _platform_wallet) = setup_contract(&env);
    let organizer = Address::generate(&env);
    let user = Address::generate(&env);
    let event_id = String::from_str(&env, "event_no_payment_test");
    
    // Register an event
    register_test_event(&env, &client, &event_id, &organizer);
    
    // Drain setup events
    let _ = env.events().all();
    
    // Join waitlist
    client.join_waitlist(&event_id, &user);
    
    // Count events after join
    let events_after = env.events().all();
    
    // Only one new event should be emitted (WaitlistJoined)
    // No token transfer events should be present
    assert_eq!(events_after.len(), 1, "expected only WaitlistJoined event, no token transfers");
}
