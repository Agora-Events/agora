use super::*;
use crate::error::EventRegistryError;
use crate::types::EventStatus;
use crate::types::{EventInfo, EventRegistrationArgs, TicketTier};
use soroban_sdk::{
    testutils::{Address as _, EnvTestConfig, Events, Ledger},
    Address, Env, Map, String,
};

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
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500);

    let organizer = Address::generate(&env);
    let payment_address = Address::generate(&env);
    let event_id = String::from_str(&env, "event_123");

    let tiers = Map::new(&env);
    let event_info = EventInfo {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: payment_address.clone(),
        platform_fee_percent: 5,
        is_active: true,
        status: EventStatus::Active,
        created_at: env.ledger().timestamp(),
        metadata_cid: String::from_str(
            &env,
            "bafkreifh22222222222222222222222222222222222222222222222222",
        ),
        max_supply: 100,
        current_supply: 0,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        is_postponed: false,
        grace_period_end: 0,
    };

    client.store_event(&event_info);

    assert!(client.event_exists(&event_id));

    let stored_event = client.get_event(&event_id).unwrap();
    assert_eq!(stored_event.event_id, event_id);
    assert_eq!(stored_event.organizer_address, organizer);
    assert_eq!(stored_event.payment_address, payment_address);
    assert_eq!(stored_event.platform_fee_percent, 5);
    assert!(stored_event.is_active);
    assert_eq!(stored_event.max_supply, 100);
    assert_eq!(stored_event.current_supply, 0);

    let fake_id = String::from_str(&env, "fake");
    assert!(!client.event_exists(&fake_id));
    assert!(client.get_event(&fake_id).is_none());
}

#[test]
fn test_organizer_events_list() {
    let env = Env::default();
    env.mock_all_auths();
    let organizer = Address::generate(&env);
    let payment_address = Address::generate(&env);

    let tiers = Map::new(&env);

    let event_1 = EventInfo {
        event_id: String::from_str(&env, "e1"),
        organizer_address: organizer.clone(),
        payment_address: payment_address.clone(),
        platform_fee_percent: 5,
        is_active: true,
        status: EventStatus::Active,
        created_at: 100,
        metadata_cid: String::from_str(
            &env,
            "bafkreifh22222222222222222222222222222222222222222222222222",
        ),
        max_supply: 50,
        current_supply: 0,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        is_postponed: false,
        grace_period_end: 0,
    };

    let event_2 = EventInfo {
        event_id: String::from_str(&env, "e2"),
        organizer_address: organizer.clone(),
        payment_address: payment_address.clone(),
        platform_fee_percent: 5,
        is_active: true,
        status: EventStatus::Active,
        created_at: 200,
        metadata_cid: String::from_str(
            &env,
            "bafkreifh22222222222222222222222222222222222222222222222222",
        ),
        max_supply: 0,
        current_supply: 0,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        is_postponed: false,
        grace_period_end: 0,
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

    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr.clone(),
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    let payment_info = client.get_event_payment_info(&event_id);
    assert_eq!(payment_info.payment_address, payment_addr);
    assert_eq!(payment_info.platform_fee_percent, 500);
    assert_eq!(payment_info.tiers.len(), 1);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.max_supply, 100);
    assert_eq!(event_info.current_supply, 0);
    assert!(!event_info.is_postponed);
    assert_eq!(event_info.grace_period_end, 0);
}

#[test]
fn test_register_event_unlimited_supply() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "unlimited_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 0,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.max_supply, 0);
    assert_eq!(event_info.current_supply, 0);
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
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: payment_addr.clone(),
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id,
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });
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
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr.clone(),
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

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
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });
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
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });
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
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: payment_addr.clone(),
        metadata_cid,
        max_supply: 200,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

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
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

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
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    let wrong_char_cid = String::from_str(
        &env,
        "Qafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let result_wrong_char = client.try_update_metadata(&event_id, &wrong_char_cid);
    assert_eq!(
        result_wrong_char,
        Err(Ok(EventRegistryError::InvalidMetadataCid))
    );

    let short_cid = String::from_str(&env, "bafy");
    let result = client.try_update_metadata(&event_id, &short_cid);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidMetadataCid)));
}

// ==================== Inventory / Supply Tests ====================

#[test]
fn test_set_ticket_payment_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    assert_eq!(client.get_ticket_payment_contract(), ticket_payment);
}

#[test]
fn test_increment_inventory_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "supply_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    let tier_id = String::from_str(&env, "general");
    tiers.set(
        tier_id.clone(),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 10,
            current_sold: 0,
            is_refundable: true,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 10,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    client.increment_inventory(&event_id, &tier_id, &1);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.current_supply, 1);
    assert_eq!(event_info.max_supply, 10);
    let tier = event_info.tiers.get(tier_id.clone()).unwrap();
    assert_eq!(tier.current_sold, 1);

    client.increment_inventory(&event_id, &tier_id, &1);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.current_supply, 2);
    let tier = event_info.tiers.get(tier_id).unwrap();
    assert_eq!(tier.current_sold, 2);
}

#[test]
fn test_increment_inventory_max_supply_exceeded() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "limited_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    let tier_id = String::from_str(&env, "general");
    tiers.set(
        tier_id.clone(),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 2,
            current_sold: 0,
            is_refundable: true,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 2,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    client.increment_inventory(&event_id, &tier_id, &1);
    client.increment_inventory(&event_id, &tier_id, &1);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.current_supply, 2);
    assert_eq!(event_info.max_supply, 2);

    let result = client.try_increment_inventory(&event_id, &tier_id, &1);
    assert_eq!(result, Err(Ok(EventRegistryError::MaxSupplyExceeded)));
}

#[test]
fn test_increment_inventory_unlimited_supply() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "unlimited_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    let tier_id = String::from_str(&env, "general");
    tiers.set(
        tier_id.clone(),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 1000,
            current_sold: 0,
            is_refundable: true,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 0,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    for _ in 0..10 {
        client.increment_inventory(&event_id, &tier_id, &1);
    }

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.current_supply, 10);
    assert_eq!(event_info.max_supply, 0);
}

#[test]
fn test_increment_inventory_event_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let fake_event_id = String::from_str(&env, "nonexistent");
    let tier_id = String::from_str(&env, "general");
    let result = client.try_increment_inventory(&fake_event_id, &tier_id, &1);
    assert_eq!(result, Err(Ok(EventRegistryError::EventNotFound)));
}

#[test]
fn test_increment_inventory_inactive_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "inactive_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let mut tiers = Map::new(&env);
    let tier_id = String::from_str(&env, "general");
    tiers.set(
        tier_id.clone(),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
        },
    );
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    client.update_event_status(&event_id, &false);

    let result = client.try_increment_inventory(&event_id, &tier_id, &1);
    assert_eq!(result, Err(Ok(EventRegistryError::EventInactive)));
}

#[test]
fn test_increment_inventory_persists_across_reads() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "persist_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let mut tiers = Map::new(&env);
    let tier_id = String::from_str(&env, "general");
    tiers.set(
        tier_id.clone(),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 50,
            current_sold: 0,
            is_refundable: true,
        },
    );
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    for _ in 0..5 {
        client.increment_inventory(&event_id, &tier_id, &1);
    }

    let event_info_1 = client.get_event(&event_id).unwrap();
    let event_info_2 = client.get_event(&event_id).unwrap();
    assert_eq!(event_info_1.current_supply, 5);
    assert_eq!(event_info_2.current_supply, 5);
    assert_eq!(event_info_1.max_supply, 50);
}

// ==================== Tiered Pricing Tests ====================

#[test]
fn test_tier_limit_exceeds_max_supply() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "tier_test");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 60,
            current_sold: 0,
            is_refundable: true,
        },
    );
    tiers.set(
        String::from_str(&env, "vip"),
        TicketTier {
            name: String::from_str(&env, "VIP"),
            price: 10000000,
            tier_limit: 50,
            current_sold: 0,
            is_refundable: true,
        },
    );

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });
    assert_eq!(
        result,
        Err(Ok(EventRegistryError::TierLimitExceedsMaxSupply))
    );
}

#[test]
fn test_tier_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "tier_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    let wrong_tier_id = String::from_str(&env, "nonexistent");
    let result = client.try_increment_inventory(&event_id, &wrong_tier_id, &1);
    assert_eq!(result, Err(Ok(EventRegistryError::TierNotFound)));
}

#[test]
fn test_tier_supply_exceeded() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "tier_limit_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    let tier_id = String::from_str(&env, "vip");
    tiers.set(
        tier_id.clone(),
        TicketTier {
            name: String::from_str(&env, "VIP"),
            price: 10000000,
            tier_limit: 3,
            current_sold: 0,
            is_refundable: true,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    client.increment_inventory(&event_id, &tier_id, &1);
    client.increment_inventory(&event_id, &tier_id, &1);
    client.increment_inventory(&event_id, &tier_id, &1);

    let result = client.try_increment_inventory(&event_id, &tier_id, &1);
    assert_eq!(result, Err(Ok(EventRegistryError::TierSupplyExceeded)));
}

#[test]
fn test_multiple_tiers_inventory() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "multi_tier_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    let general_id = String::from_str(&env, "general");
    let vip_id = String::from_str(&env, "vip");

    tiers.set(
        general_id.clone(),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 50,
            current_sold: 0,
            is_refundable: true,
        },
    );
    tiers.set(
        vip_id.clone(),
        TicketTier {
            name: String::from_str(&env, "VIP"),
            price: 10000000,
            tier_limit: 20,
            current_sold: 0,
            is_refundable: true,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 70,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    client.increment_inventory(&event_id, &general_id, &1);
    client.increment_inventory(&event_id, &general_id, &1);
    client.increment_inventory(&event_id, &vip_id, &1);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.current_supply, 3);

    let general_tier = event_info.tiers.get(general_id).unwrap();
    assert_eq!(general_tier.current_sold, 2);

    let vip_tier = event_info.tiers.get(vip_id).unwrap();
    assert_eq!(vip_tier.current_sold, 1);
}

#[test]
fn test_update_event_status_noop_skips_event() {
    let env = Env::new_with_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "status_noop_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    let _ = env.events().all();
    client.update_event_status(&event_id, &true);
    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn test_blacklist_organizer() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let organizer = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let reason = String::from_str(&env, "Fraudulent activity detected");
    client.blacklist_organizer(&organizer, &reason);

    assert!(client.is_organizer_blacklisted(&organizer));

    let audit_log = client.get_blacklist_audit_log();
    assert_eq!(audit_log.len(), 1);

    let audit_entry = audit_log.get(0).unwrap();
    assert!(audit_entry.added_to_blacklist);
    assert_eq!(audit_entry.organizer_address, organizer);
    assert_eq!(audit_entry.admin_address, admin);
    assert_eq!(audit_entry.reason, reason);
}

#[test]
fn test_blacklist_prevents_event_registration() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let reason = String::from_str(&env, "Suspicious activity");
    client.blacklist_organizer(&organizer, &reason);

    let event_id = String::from_str(&env, "test_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id,
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    assert_eq!(result, Err(Ok(EventRegistryError::OrganizerBlacklisted)));
}

#[test]
fn test_update_metadata_noop_skips_event() {
    let env = Env::new_with_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "metadata_noop_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: payment_addr,
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    let _ = env.events().all();
    client.update_metadata(&event_id, &metadata_cid);
    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn test_remove_from_blacklist() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let organizer = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    // Blacklist organizer
    let reason = String::from_str(&env, "Initial blacklist");
    client.blacklist_organizer(&organizer, &reason);
    assert!(client.is_organizer_blacklisted(&organizer));

    // Remove from blacklist
    let removal_reason = String::from_str(&env, "Investigation completed");
    client.remove_from_blacklist(&organizer, &removal_reason);

    // Verify organizer is no longer blacklisted
    assert!(!client.is_organizer_blacklisted(&organizer));

    // Verify audit log has both entries
    let audit_log = client.get_blacklist_audit_log();
    assert_eq!(audit_log.len(), 2);

    // First entry - addition
    let add_entry = audit_log.get(0).unwrap();
    assert!(add_entry.added_to_blacklist);

    // Second entry - removal
    let remove_entry = audit_log.get(1).unwrap();
    assert!(!remove_entry.added_to_blacklist);
    assert_eq!(remove_entry.reason, removal_reason);
}

#[test]
fn test_blacklist_suspends_active_events() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "test_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: payment_addr,
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    let event_info = client.get_event(&event_id).unwrap();
    assert!(event_info.is_active);

    let reason = String::from_str(&env, "Fraud detected");
    client.blacklist_organizer(&organizer, &reason);

    let event_info = client.get_event(&event_id).unwrap();
    assert!(!event_info.is_active);
}

#[test]
#[should_panic] // Authentication failure
fn test_blacklist_unauthorized_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let organizer = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    // Try to blacklist organizer without admin auth - should panic
    let reason = String::from_str(&env, "Malicious attempt");
    client.blacklist_organizer(&organizer, &reason);
}

#[test]
fn test_double_blacklist_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let organizer = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    // Blacklist organizer once
    let reason = String::from_str(&env, "First blacklist");
    client.blacklist_organizer(&organizer, &reason);

    // Try to blacklist again - should fail
    let reason2 = String::from_str(&env, "Second blacklist");
    let result = client.try_blacklist_organizer(&organizer, &reason2);
    assert_eq!(result, Err(Ok(EventRegistryError::OrganizerBlacklisted)));
}

#[test]
fn test_remove_non_blacklisted_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let organizer = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    // Try to remove non-blacklisted organizer - should fail
    let reason = String::from_str(&env, "Removal attempt");
    let result = client.try_remove_from_blacklist(&organizer, &reason);
    assert_eq!(result, Err(Ok(EventRegistryError::OrganizerNotBlacklisted)));
}

// ==================== Resale Cap Tests ====================

#[test]
fn test_register_event_with_resale_cap() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "capped_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: Some(1000), // 10% above face value
    });

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.resale_cap_bps, Some(1000));
}

#[test]
fn test_register_event_resale_cap_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "no_markup_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: Some(0), // No markup allowed
    });

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.resale_cap_bps, Some(0));
}

#[test]
fn test_register_event_resale_cap_none() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "free_market_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None, // No cap
    });

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.resale_cap_bps, None);
}

#[test]
fn test_postpone_event_sets_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "postponed_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    // Set ledger time and grace period end in the future
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let grace_period_end = 2_000u64;

    client.postpone_event(&event_id, &grace_period_end);

    let event_info = client.get_event(&event_id).unwrap();
    assert!(event_info.is_postponed);
    assert_eq!(event_info.grace_period_end, grace_period_end);
}

#[test]
fn test_register_event_resale_cap_invalid() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "bad_cap_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id,
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: Some(10001), // Over 100% - invalid
    });
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidResaleCapBps)));
}

#[test]
fn test_cancel_event_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "cancel_me");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 100,
        resale_cap_bps: None,
    });

    client.cancel_event(&event_id);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.status, EventStatus::Cancelled);
    assert!(!event_info.is_active);
}

#[test]
fn test_cancel_already_cancelled_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "cancel_twice");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: Address::generate(&env),
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    client.cancel_event(&event_id);
    let result = client.try_cancel_event(&event_id);
    assert_eq!(result, Err(Ok(EventRegistryError::EventAlreadyCancelled)));
}

#[test]
fn test_update_status_on_cancelled_event_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500);

    let event_id = String::from_str(&env, "no_updates");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        organizer_address: organizer.clone(),
        payment_address: Address::generate(&env),
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
    });

    client.cancel_event(&event_id);
    let result = client.try_update_event_status(&event_id, &true);
    assert_eq!(result, Err(Ok(EventRegistryError::EventCancelled)));
}
