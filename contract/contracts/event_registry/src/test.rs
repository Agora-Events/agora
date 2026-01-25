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
#[should_panic(expected = "Fee percent must be between 0 and 10000")]
fn test_initialization_invalid_fee() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &10001); // Should panic
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
#[should_panic(expected = "Fee percent must be between 0 and 10000")]
fn test_set_platform_fee_invalid() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin, &5);
    client.set_platform_fee(&10001); // Should panic
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

// Event Registration Tests
#[test]
fn test_register_event_success() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);

    // Mock authorization for all addresses
    env.mock_all_auths();

    // Initialize contract
    client.initialize(&admin, &500);

    // Register event
    let event_id = String::from_str(&env, "event_001");
    client.register_event(&event_id, &organizer, &payment_addr);

    // Verify event was registered
    let payment_info = client.get_event_payment_info(&event_id);
    assert_eq!(payment_info.payment_address, payment_addr);
    assert_eq!(payment_info.platform_fee_percent, 500);

    // Verify event exists
    assert!(client.event_exists(&event_id));
}

#[test]
#[should_panic(expected = "Event already exists")]
fn test_register_duplicate_event_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &500);

    let event_id = String::from_str(&env, "event_001");

    // Register event first time
    client.register_event(&event_id, &organizer, &payment_addr);

    // Try to register same event again - should panic
    client.register_event(&event_id, &organizer, &payment_addr);
}

#[test]
fn test_get_event_payment_info() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &750); // 7.5%

    let event_id = String::from_str(&env, "event_002");
    client.register_event(&event_id, &organizer, &payment_addr);

    // Test successful query
    let info = client.get_event_payment_info(&event_id);
    assert_eq!(info.payment_address, payment_addr);
    assert_eq!(info.platform_fee_percent, 750);
}

#[test]
#[should_panic(expected = "Event not found")]
fn test_get_nonexistent_event_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    // Test query for non-existent event - should panic
    let non_existent_id = String::from_str(&env, "non_existent");
    client.get_event_payment_info(&non_existent_id);
}

#[test]
fn test_update_event_status() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &500);

    let event_id = String::from_str(&env, "event_001");
    client.register_event(&event_id, &organizer, &payment_addr);

    // Test successful status update by organizer
    client.update_event_status(&event_id, &false);

    // Verify the event was updated
    let event_info = client.get_event(&event_id).unwrap();
    assert!(!event_info.is_active);
}

#[test]
#[should_panic(expected = "Event not found")]
fn test_update_nonexistent_event_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    // Test update for non-existent event - should panic
    let non_existent_id = String::from_str(&env, "non_existent");
    client.update_event_status(&non_existent_id, &false);
}

#[test]
fn test_complete_event_lifecycle() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin, &600); // 6%

    let event_id = String::from_str(&env, "lifecycle_event");

    // 1. Register event
    client.register_event(&event_id, &organizer, &payment_addr);

    // 2. Query payment info
    let payment_info = client.get_event_payment_info(&event_id);
    assert_eq!(payment_info.payment_address, payment_addr);
    assert_eq!(payment_info.platform_fee_percent, 600);

    // 3. Check organizer events
    let org_events = client.get_organizer_events(&organizer);
    assert_eq!(org_events.len(), 1);
    assert!(org_events.contains(&event_id));

    // 4. Update event status
    client.update_event_status(&event_id, &false);

    // 5. Verify event still exists and can be queried
    let payment_info = client.get_event_payment_info(&event_id);
    assert_eq!(payment_info.payment_address, payment_addr);

    // 6. Verify event info shows updated status
    let event_info = client.get_event(&event_id).unwrap();
    assert!(!event_info.is_active);
}
