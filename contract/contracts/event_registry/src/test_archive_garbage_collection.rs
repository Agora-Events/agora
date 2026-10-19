//! # Archive & Garbage Collection Tests
//!
//! This module tests the garbage collection feature for old event data.
//! The archive_event function enforces a 30-day waiting period after event_end_time
//! before allowing archival, which reclaims storage by deleting non-essential data.

use crate::{
    error::EventRegistryError,
    types::{EventRegistrationArgs, TicketTier},
    EventRegistry, EventRegistryClient,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Env, Map, String,
};

/// Helper function to create a test environment with initialized contract
fn setup_test_env() -> (Env, EventRegistryClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    (env, client, admin, platform_wallet, usdc_token)
}

/// Helper function to register a test event with end_time
fn register_test_event(
    env: &Env,
    client: &EventRegistryClient,
    event_id: String,
    organizer: &Address,
    end_time: u64,
) {
    let mut tiers = Map::new(env);
    tiers.set(
        String::from_str(env, "general"),
        TicketTier {
            name: String::from_str(env, "General Admission"),
            price: 1000_0000000, // 1000 USDC
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
            auction_config: vec![env],
            loyalty_multiplier: 1,
            max_per_user: 0,
        },
    );

    let args = EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(env, "Test Event for Archival"),
        organizer_address: organizer.clone(),
        payment_address: organizer.clone(),
        metadata_cid: String::from_str(
            env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: Some(vec![
            env,
            String::from_str(env, "music"),
            String::from_str(env, "festival"),
        ]),
        category_ids: Some(vec![env, 1, 2]), // Music, Sports
        start_time: 0,
        is_private: false,
        end_time,
        transfer_lock_duration: 0,
        accepted_tokens: vec![env],
        use_global_whitelist: true,
        referral_rate_bps: None,
    };

    client.register_event(&args);
}

#[test]
fn test_archive_event_success_after_30_days() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_archive_success");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event with end_time set to initial_time + 1 day
    let event_end_time = initial_time + (24 * 60 * 60); // 1 day later
    register_test_event(&env, &client, event_id.clone(), &organizer, event_end_time);

    // Deactivate the event (required for archival)
    client.update_event_status(&event_id, &false);

    // Verify event exists before archival
    let event_before = client.get_event(&event_id);
    assert!(event_before.is_some());
    let event_info = event_before.unwrap();
    assert_eq!(event_info.end_time, event_end_time);
    assert!(!event_info.is_active);

    // Fast forward time to 30 days + 1 second after end_time
    let grace_period = 30 * 24 * 60 * 60; // 30 days in seconds
    let archive_time = event_end_time + grace_period + 1;
    env.ledger().set(LedgerInfo {
        timestamp: archive_time,
        protocol_version: 23,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Archive the event
    let result = client.try_archive_event(&event_id);
    assert!(result.is_ok());

    // Verify event is removed (garbage collected)
    let event_after = client.get_event(&event_id);
    assert!(event_after.is_none());

    // Verify receipt exists with minimal data
    let receipt = client.get_organizer_receipts(&organizer);
    assert_eq!(receipt.len(), 1);
    assert_eq!(receipt.get(0).unwrap().event_id, event_id);
    assert_eq!(receipt.get(0).unwrap().organizer_address, organizer);
    assert_eq!(receipt.get(0).unwrap().total_sold, 0);
    assert_eq!(receipt.get(0).unwrap().archived_at, archive_time);
}

#[test]
fn test_archive_event_fails_before_30_days() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_archive_too_early");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event with end_time
    let event_end_time = initial_time + (24 * 60 * 60); // 1 day later
    register_test_event(&env, &client, event_id.clone(), &organizer, event_end_time);

    // Deactivate the event
    client.update_event_status(&event_id, &false);

    // Try to archive after only 29 days (should fail)
    let too_early_time = event_end_time + (29 * 24 * 60 * 60);
    env.ledger().set(LedgerInfo {
        timestamp: too_early_time,
        protocol_version: 23,
        sequence_number: 50,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let result = client.try_archive_event(&event_id);
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Ok(EventRegistryError::InvalidDeadline)));

    // Verify event still exists (not archived)
    let event_after = client.get_event(&event_id);
    assert!(event_after.is_some());
}

#[test]
fn test_archive_event_fails_if_active() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_archive_active");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event with end_time
    let event_end_time = initial_time + (24 * 60 * 60);
    register_test_event(&env, &client, event_id.clone(), &organizer, event_end_time);

    // Event is still active - do NOT deactivate

    // Fast forward time to 30 days after end_time
    let archive_time = event_end_time + (30 * 24 * 60 * 60) + 1;
    env.ledger().set(LedgerInfo {
        timestamp: archive_time,
        protocol_version: 23,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Try to archive (should fail because event is still active)
    let result = client.try_archive_event(&event_id);
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Ok(EventRegistryError::EventIsActive)));

    // Verify event still exists
    let event_after = client.get_event(&event_id);
    assert!(event_after.is_some());
    assert!(event_after.unwrap().is_active);
}

#[test]
fn test_archive_event_fails_if_no_end_time() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_archive_no_end_time");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event WITHOUT end_time (end_time = 0)
    register_test_event(&env, &client, event_id.clone(), &organizer, 0);

    // Deactivate the event
    client.update_event_status(&event_id, &false);

    // Fast forward time significantly
    let future_time = initial_time + (365 * 24 * 60 * 60); // 1 year later
    env.ledger().set(LedgerInfo {
        timestamp: future_time,
        protocol_version: 23,
        sequence_number: 1000,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Try to archive (should fail because end_time is not set)
    let result = client.try_archive_event(&event_id);
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Ok(EventRegistryError::EventNotEnded)));

    // Verify event still exists
    let event_after = client.get_event(&event_id);
    assert!(event_after.is_some());
}

#[test]
fn test_archive_event_fails_if_not_found() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let event_id = String::from_str(&env, "nonexistent_event");

    let result = client.try_archive_event(&event_id);
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Ok(EventRegistryError::EventNotFound)));
}

#[test]
fn test_archive_event_exactly_at_30_days() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_archive_exact_30_days");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event with end_time
    let event_end_time = initial_time + (24 * 60 * 60);
    register_test_event(&env, &client, event_id.clone(), &organizer, event_end_time);

    // Deactivate the event
    client.update_event_status(&event_id, &false);

    // Try to archive exactly at 30 days (should fail - needs to be AFTER 30 days)
    let exact_30_days = event_end_time + (30 * 24 * 60 * 60);
    env.ledger().set(LedgerInfo {
        timestamp: exact_30_days,
        protocol_version: 23,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let result = client.try_archive_event(&event_id);
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Ok(EventRegistryError::InvalidDeadline)));
}

#[test]
fn test_archive_event_with_sold_tickets() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    let buyer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_archive_with_sales");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event with end_time
    let event_end_time = initial_time + (24 * 60 * 60);
    register_test_event(&env, &client, event_id.clone(), &organizer, event_end_time);

    // Set up ticket payment contract
    client.set_ticket_payment_contract(&ticket_payment);

    // Simulate ticket sales
    client.increment_inventory(&event_id, &String::from_str(&env, "general"), &5);

    // Verify tickets were sold
    let event_before = client.get_event(&event_id).unwrap();
    assert_eq!(event_before.current_supply, 5);

    // Deactivate the event
    client.update_event_status(&event_id, &false);

    // Fast forward time to 30 days + 1 second after end_time
    let archive_time = event_end_time + (30 * 24 * 60 * 60) + 1;
    env.ledger().set(LedgerInfo {
        timestamp: archive_time,
        protocol_version: 23,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Archive the event
    let result = client.try_archive_event(&event_id);
    assert!(result.is_ok());

    // Verify receipt preserves the total_sold count
    let receipt = client.get_organizer_receipts(&organizer);
    assert_eq!(receipt.len(), 1);
    assert_eq!(receipt.get(0).unwrap().total_sold, 5);
}

#[test]
fn test_multiple_events_archival() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register multiple events
    let event_id_1 = String::from_str(&env, "event_1");
    let event_id_2 = String::from_str(&env, "event_2");
    let event_id_3 = String::from_str(&env, "event_3");

    let event_end_time = initial_time + (24 * 60 * 60);
    register_test_event(
        &env,
        &client,
        event_id_1.clone(),
        &organizer,
        event_end_time,
    );
    register_test_event(
        &env,
        &client,
        event_id_2.clone(),
        &organizer,
        event_end_time,
    );
    register_test_event(
        &env,
        &client,
        event_id_3.clone(),
        &organizer,
        event_end_time,
    );

    // Deactivate all events
    client.update_event_status(&event_id_1, &false);
    client.update_event_status(&event_id_2, &false);
    client.update_event_status(&event_id_3, &false);

    // Fast forward time to 30 days + 1 second after end_time
    let archive_time = event_end_time + (30 * 24 * 60 * 60) + 1;
    env.ledger().set(LedgerInfo {
        timestamp: archive_time,
        protocol_version: 23,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Archive all events
    assert!(client.try_archive_event(&event_id_1).is_ok());
    assert!(client.try_archive_event(&event_id_2).is_ok());
    assert!(client.try_archive_event(&event_id_3).is_ok());

    // Verify all events are archived
    assert!(client.get_event(&event_id_1).is_none());
    assert!(client.get_event(&event_id_2).is_none());
    assert!(client.get_event(&event_id_3).is_none());

    // Verify all receipts exist
    let receipts = client.get_organizer_receipts(&organizer);
    assert_eq!(receipts.len(), 3);
}


// Issue #684: Add event_registry unit test for archive_event storage reclamation

#[test]
fn test_archive_removes_full_event_info() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_storage_reclaim");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event with end_time
    let event_end_time = initial_time + (24 * 60 * 60);
    register_test_event(&env, &client, event_id.clone(), &organizer, event_end_time);

    // Simulate some ticket sales
    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);
    let buyer = Address::generate(&env);
    client.increment_inventory(&event_id, &String::from_str(&env, "general"), &10);

    // Verify full EventInfo exists before archival
    let event_before = client.get_event(&event_id);
    assert!(event_before.is_some());
    let event_info = event_before.unwrap();
    assert_eq!(event_info.current_supply, 10);
    assert!(event_info.tiers.len() > 0);
    assert!(event_info.tags.is_some());

    // Deactivate the event
    client.update_event_status(&event_id, &false);

    // Advance ledger past end_time + 30 days
    let archive_time = event_end_time + (30 * 24 * 60 * 60) + 1;
    env.ledger().set(LedgerInfo {
        timestamp: archive_time,
        protocol_version: 23,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Call archive_event
    let result = client.try_archive_event(&event_id);
    assert!(result.is_ok());

    // Assert get_event returns None (full info removed)
    let event_after = client.get_event(&event_id);
    assert!(event_after.is_none(), "Full EventInfo should be removed after archiving");

    // Assert get_event_receipt returns Some(EventReceipt) with correct fields
    let receipts = client.get_organizer_receipts(&organizer);
    assert_eq!(receipts.len(), 1);
    
    let receipt = receipts.get(0).unwrap();
    assert_eq!(receipt.event_id, event_id);
    assert_eq!(receipt.organizer_address, organizer);
    assert_eq!(receipt.total_sold, 10);
    assert_eq!(receipt.archived_at, archive_time);
}

#[test]
fn test_archive_before_end_time_rejected() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_archive_before_end");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event with end_time in the future
    let event_end_time = initial_time + (365 * 24 * 60 * 60); // 1 year later
    register_test_event(&env, &client, event_id.clone(), &organizer, event_end_time);

    // Deactivate the event
    client.update_event_status(&event_id, &false);

    // Try to archive before event has ended (current time < end_time)
    // Current time is still initial_time, which is before event_end_time
    let result = client.try_archive_event(&event_id);
    
    // Assert EventNotEnded error
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Ok(EventRegistryError::EventNotEnded)));

    // Verify event still exists
    let event_after = client.get_event(&event_id);
    assert!(event_after.is_some());
}

#[test]
fn test_archive_active_event_rejected() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_archive_active_rejected");

    // Set initial ledger time
    let initial_time = 1_000_000u64;
    env.ledger().set(LedgerInfo {
        timestamp: initial_time,
        protocol_version: 23,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Register event with end_time
    let event_end_time = initial_time + (24 * 60 * 60);
    register_test_event(&env, &client, event_id.clone(), &organizer, event_end_time);

    // Event is still active - do NOT deactivate

    // Advance ledger past end_time + 30 days
    let archive_time = event_end_time + (30 * 24 * 60 * 60) + 1;
    env.ledger().set(LedgerInfo {
        timestamp: archive_time,
        protocol_version: 23,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Try to archive while event is still active (not yet 30 days past end)
    let result = client.try_archive_event(&event_id);
    
    // Assert appropriate error (EventIsActive)
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Ok(EventRegistryError::EventIsActive)));

    // Verify event still exists
    let event_after = client.get_event(&event_id);
    assert!(event_after.is_some());
    assert!(event_after.unwrap().is_active);
}
