use super::*;
use crate::error::EventRegistryError;
use crate::types::EventInfo;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &0);

    assert_eq!(client.get_platform_fee(), 500);
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_platform_wallet(), platform_wallet);
}

#[test]
fn test_double_initialization_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    let result = client.try_initialize(&admin, &platform_wallet, &1000);
    assert_eq!(result, Err(Ok(EventRegistryError::AlreadyInitialized)));
}

#[test]
fn test_initialization_invalid_fee() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    let result = client.try_initialize(&admin, &platform_wallet, &10001);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidFeePercent)));
}

#[test]
fn test_initialization_invalid_address() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let contract_address = client.address.clone();
    let platform_wallet = Address::generate(&env);

    let result = client.try_initialize(&contract_address, &platform_wallet, &500);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidAddress)));
}

#[test]
fn test_set_platform_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_platform_fee(&10);

    assert_eq!(client.get_platform_fee(), 10);
}

#[test]
fn test_set_platform_fee_invalid() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    let result = client.try_set_platform_fee(&10001);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidFeePercent)));
}

#[test]
#[should_panic] // Authentication failure
fn test_set_platform_fee_unauthorized() {
    let env = Env::default();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_platform_fee(&10);
}

#[test]
fn test_storage_operations() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500);

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
        metadata_cid: String::from_str(
            &env,
            "bafkreifh22222222222222222222222222222222222222222222222222",
        ),
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
        metadata_cid: String::from_str(
            &env,
            "bafkreifh22222222222222222222222222222222222222222222222222",
        ),
    };

    let event_2 = EventInfo {
        event_id: String::from_str(&env, "e2"),
        organizer_address: organizer.clone(),
        payment_address: payment_address.clone(),
        platform_fee_percent: 5,
        is_active: true,
        created_at: 200,
        metadata_cid: String::from_str(
            &env,
            "bafkreifh22222222222222222222222222222222222222222222222222",
        ),
    };

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    client.store_event(&event_1);
    client.store_event(&event_2);

    let organizer_events = client.get_organizer_events(&organizer);
    assert_eq!(organizer_events.len(), 2);
    assert_eq!(organizer_events.get(0).unwrap(), event_1.event_id);
    assert_eq!(organizer_events.get(1).unwrap(), event_2.event_id);
}

#[test]
fn test_register_event_success() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "event_001");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&event_id, &organizer, &payment_addr, &metadata_cid);

    let payment_info = client.get_event_payment_info(&event_id);
    assert_eq!(payment_info.payment_address, payment_addr);
    assert_eq!(payment_info.platform_fee_percent, 500);
}

#[test]
fn test_register_duplicate_event_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "event_001");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&event_id, &organizer, &payment_addr, &metadata_cid);

    let result = client.try_register_event(&event_id, &organizer, &payment_addr, &metadata_cid);
    assert_eq!(result, Err(Ok(EventRegistryError::EventAlreadyExists)));
}

#[test]
fn test_get_event_payment_info() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &platform_wallet, &750);

    let event_id = String::from_str(&env, "event_002");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&event_id, &organizer, &payment_addr, &metadata_cid);

    let info = client.get_event_payment_info(&event_id);
    assert_eq!(info.payment_address, payment_addr);
    assert_eq!(info.platform_fee_percent, 750);
}

#[test]
fn test_update_event_status() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "event_001");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&event_id, &organizer, &payment_addr, &metadata_cid);
    client.update_event_status(&event_id, &false);

    let event_info = client.get_event(&event_id).unwrap();
    assert!(!event_info.is_active);
}

#[test]
fn test_event_inactive_error() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &platform_wallet, &500);
    let event_id = String::from_str(&env, "event_001");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&event_id, &organizer, &payment_addr, &metadata_cid);
    client.update_event_status(&event_id, &false);

    let result = client.try_get_event_payment_info(&event_id);
    assert_eq!(result, Err(Ok(EventRegistryError::EventInactive)));
}

#[test]
fn test_complete_event_lifecycle() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &platform_wallet, &600);

    let event_id = String::from_str(&env, "lifecycle_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&event_id, &organizer, &payment_addr, &metadata_cid);

    let payment_info = client.get_event_payment_info(&event_id);
    assert_eq!(payment_info.payment_address, payment_addr);
    assert_eq!(payment_info.platform_fee_percent, 600);

    let org_events = client.get_organizer_events(&organizer);
    assert_eq!(org_events.len(), 1);
    assert!(org_events.contains(&event_id));

    client.update_event_status(&event_id, &false);

    let result = client.try_get_event_payment_info(&event_id);
    assert_eq!(result, Err(Ok(EventRegistryError::EventInactive)));

    let event_info = client.get_event(&event_id).unwrap();
    assert!(!event_info.is_active);
}

#[test]
fn test_update_metadata_success() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "event_metadata");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&event_id, &organizer, &payment_addr, &metadata_cid);

    let new_metadata_cid = String::from_str(
        &env,
        "bafkreifh22222222222222222222222222222222222222222222222222",
    );
    client.update_metadata(&event_id, &new_metadata_cid);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.metadata_cid, new_metadata_cid);
}

#[test]
fn test_update_metadata_invalid_cid() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "event_metadata");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&event_id, &organizer, &payment_addr, &metadata_cid);

    // Test starts with wrong character
    let wrong_char_cid = String::from_str(
        &env,
        "Qafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let result_wrong_char = client.try_update_metadata(&event_id, &wrong_char_cid);
    assert_eq!(
        result_wrong_char,
        Err(Ok(EventRegistryError::InvalidMetadataCid))
    );

    // Test too short
    let short_cid = String::from_str(&env, "bafy");
    let result = client.try_update_metadata(&event_id, &short_cid);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidMetadataCid)));
}
