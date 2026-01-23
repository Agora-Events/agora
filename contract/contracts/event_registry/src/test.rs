#![cfg(test)]

use super::*;
use crate::types::EventInfo;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &5);

    assert_eq!(client.get_platform_fee(), 5);
    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialization_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &5);
    client.initialize(&admin, &10); // Should panic
}

#[test]
#[should_panic(expected = "Fee percent must be between 0 and 100")]
fn test_initialization_invalid_fee() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &101); // Should panic
}

#[test]
fn test_set_platform_fee() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &5);
    client.set_platform_fee(&10);

    assert_eq!(client.get_platform_fee(), 10);
}

#[test]
#[should_panic(expected = "Fee percent must be between 0 and 100")]
fn test_set_platform_fee_invalid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &5);
    client.set_platform_fee(&101); // Should panic
}

#[test]
#[should_panic] // Authentication failure
fn test_set_platform_fee_unauthorized() {
    let env = Env::default();
    
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &5);

    // This will fail because no auth is mocked/provided for the admin address stored in the contract
    client.set_platform_fee(&10);
}

#[test]
fn test_storage_operations() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &5);

    let organizer = Address::generate(&env);
    let payment_address = Address::generate(&env);
    let event_id = String::from_str(&env, "event_123");

    let event_info = EventInfo {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: payment_address.clone(),
        platform_fee_percent: 5,
        is_active: true,
        created_at: env.ledger().timestamp(),
    };

    // Test store_event
    client.store_event(&event_info);

    // Test event_exists
    assert!(client.event_exists(&event_id));

    // Test get_event
    let stored_event = client.get_event(&event_id).unwrap();
    assert_eq!(stored_event.event_id, event_id);
    assert_eq!(stored_event.organizer_address, organizer);
    assert_eq!(stored_event.payment_address, payment_address);
    assert_eq!(stored_event.platform_fee_percent, 5);
    assert!(stored_event.is_active);

    // Test non-existent event
    let fake_id = String::from_str(&env, "fake");
    assert!(!client.event_exists(&fake_id));
    assert!(client.get_event(&fake_id).is_none());
}

#[test]
fn test_organizer_events_list() {
    let env = Env::default();
    let organizer = Address::generate(&env);
    let payment_address = Address::generate(&env);

    let event_1 = EventInfo {
        event_id: String::from_str(&env, "e1"),
        organizer_address: organizer.clone(),
        payment_address: payment_address.clone(),
        platform_fee_percent: 5,
        is_active: true,
        created_at: 100,
    };

    let event_2 = EventInfo {
        event_id: String::from_str(&env, "e2"),
        organizer_address: organizer.clone(),
        payment_address: payment_address.clone(),
        platform_fee_percent: 5,
        is_active: true,
        created_at: 200,
    };

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    client.store_event(&event_1);
    client.store_event(&event_2);

    let event_exists_1 = client.event_exists(&event_1.event_id);
    let event_exists_2 = client.event_exists(&event_2.event_id);
    assert!(event_exists_1);
    assert!(event_exists_2);

    let organizer_events = client.get_organizer_events(&organizer);
    assert_eq!(organizer_events.len(), 2);
    assert_eq!(organizer_events.get(0).unwrap(), event_1.event_id);
    assert_eq!(organizer_events.get(1).unwrap(), event_2.event_id);
}
