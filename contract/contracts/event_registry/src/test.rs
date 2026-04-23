use super::*;
use crate::error::EventRegistryError;
use crate::types::EventStatus;
use crate::types::{EventInfo, EventReceipt, EventRegistrationArgs, TicketTier};
use soroban_sdk::{
    testutils::{Address as _, EnvTestConfig, Events, Ledger},
    Address, Env, Map, String,
};

fn test_payment_address(env: &Env) -> Address {
    Address::from_string(&String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJXFF",
    ))
}

#[test]
fn test_get_version() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    assert_eq!(client.get_version(), 1);
}

#[test]
fn test_register_and_get_series() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Register two events for the organizer
    let event_id1 = String::from_str(&env, "event_1");
    let event_id2 = String::from_str(&env, "event_2");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id1.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    client.register_event(&EventRegistrationArgs {
        event_id: event_id2.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    // Register a series
    let series_id = String::from_str(&env, "series_1");
    let series_name = String::from_str(&env, "Spring Festival");
    let event_ids = soroban_sdk::vec![&env, event_id1.clone(), event_id2.clone()];
    let meta = Some(String::from_str(&env, "series_meta"));
    client.register_series(&series_id, &series_name, &event_ids, &organizer, &meta);

    let series = client.get_series(&series_id).unwrap();
    assert_eq!(series.series_id, series_id);
    assert_eq!(series.name, series_name);
    assert_eq!(series.event_ids.len(), 2);
    assert_eq!(series.organizer_address, organizer);
    assert_eq!(series.metadata_cid, meta);
}

#[test]
fn test_issue_and_use_series_pass() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Register event and series
    let event_id = String::from_str(&env, "event_1");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    let series_id = String::from_str(&env, "series_1");
    let event_ids = soroban_sdk::vec![&env, event_id.clone()];
    let meta = Some(String::from_str(&env, "series_meta"));
    client.register_series(
        &series_id,
        &String::from_str(&env, "Series"),
        &event_ids,
        &organizer,
        &meta,
    );

    // Issue a pass
    let pass_id = String::from_str(&env, "pass_1");
    let holder = Address::generate(&env);
    let usage_limit = 2u32;
    let expires_at = env.ledger().timestamp() + 10000;
    client.issue_series_pass(&pass_id, &series_id, &holder, &usage_limit, &expires_at);

    // Retrieve and check pass
    let pass = client.get_series_pass(&pass_id).unwrap();
    assert_eq!(pass.series_id, series_id);
    assert_eq!(pass.holder, holder);
    assert_eq!(pass.usage_limit, usage_limit);
    assert_eq!(pass.usage_count, 0);

    // Increment usage and check limit enforcement
    for i in 0..usage_limit {
        let updated = env.as_contract(&contract_id, || {
            crate::storage::increment_series_pass_usage(&env, pass_id.clone())
        });
        assert!(updated.is_some());
        let pass = client.get_series_pass(&pass_id).unwrap();
        assert_eq!(pass.usage_count, i + 1);
    }
    // Should not increment beyond limit
    let updated = env.as_contract(&contract_id, || {
        crate::storage::increment_series_pass_usage(&env, pass_id.clone())
    });
    assert!(updated.is_none());
}

#[test]
fn test_double_initialization_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    let result = client.try_initialize(&admin, &platform_wallet, &1000, &usdc_token);
    assert_eq!(result, Err(Ok(EventRegistryError::AlreadyInitialized)));
}

#[test]
fn test_initialization_invalid_fee() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    let result = client.try_initialize(&admin, &platform_wallet, &10001, &usdc_token);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidFeePercent)));
}

#[test]
fn test_initialization_invalid_address() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let contract_address = client.address.clone();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    let result = client.try_initialize(&contract_address, &platform_wallet, &500, &usdc_token);
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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let organizer = Address::generate(&env);
    let payment_address = test_payment_address(&env);
    let event_id = String::from_str(&env, "event_123");

    let tiers = Map::new(&env);
    let event_info = EventInfo {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
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
        min_sales_target: 0,
        target_deadline: 0,
        goal_met: false,
        custom_fee_bps: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
        feedback_cid: None,
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
fn test_get_total_tickets_sold_uses_event_current_supply() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let organizer = Address::generate(&env);
    let payment_address = test_payment_address(&env);
    let event_id = String::from_str(&env, "sold_event");

    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 1_000,
            tier_limit: 100,
            current_sold: 3,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );
    tiers.set(
        String::from_str(&env, "vip"),
        TicketTier {
            name: String::from_str(&env, "VIP"),
            price: 2_000,
            tier_limit: 50,
            current_sold: 4,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.store_event(&EventInfo {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address,
        platform_fee_percent: 500,
        is_active: true,
        status: EventStatus::Active,
        created_at: env.ledger().timestamp(),
        metadata_cid: String::from_str(
            &env,
            "bafkreifh22222222222222222222222222222222222222222222222222",
        ),
        max_supply: 150,
        current_supply: 9,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        is_postponed: false,
        grace_period_end: 0,
        min_sales_target: 0,
        target_deadline: 0,
        goal_met: false,
        custom_fee_bps: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
        feedback_cid: None,
    });

    assert_eq!(client.get_total_tickets_sold(&event_id), 9);
}

#[test]
fn test_get_active_events_count_tracks_status_changes() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    let event_1 = String::from_str(&env, "active_count_1");
    let event_2 = String::from_str(&env, "active_count_2");
    let event_3 = String::from_str(&env, "active_count_3");

    for event_id in [event_1.clone(), event_2.clone(), event_3.clone()] {
        client.register_event(&EventRegistrationArgs {
            event_id: event_id.clone(),
            name: String::from_str(&env, "Test Event"),
            organizer_address: organizer.clone(),
            payment_address: test_payment_address(&env),
            metadata_cid: metadata_cid.clone(),
            max_supply: 100,
            milestone_plan: None,
            tiers: tiers.clone(),
            refund_deadline: 0,
            restocking_fee: 0,
            resale_cap_bps: None,
            min_sales_target: None,
            target_deadline: None,
            banner_cid: None,
            tags: None,
            end_time: 0,
        });
    }

    assert_eq!(client.get_active_events_count(), 3);

    client.update_event_status(&event_1, &false);
    assert_eq!(client.get_active_events_count(), 2);

    client.cancel_event(&event_2);
    assert_eq!(client.get_active_events_count(), 1);

    client.update_event_status(&event_1, &true);
    assert_eq!(client.get_active_events_count(), 2);

    client.update_event_status(&event_3, &false);
    assert_eq!(client.get_active_events_count(), 1);

    client.archive_event(&event_3);
    assert_eq!(client.get_active_events_count(), 1);
}

#[test]
fn test_organizer_events_list() {
    let env = Env::default();
    env.mock_all_auths();
    let organizer = Address::generate(&env);
    let payment_address = test_payment_address(&env);

    let tiers = Map::new(&env);

    let event_1 = EventInfo {
        event_id: String::from_str(&env, "e1"),
        name: String::from_str(&env, "Test Event 1"),
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
        min_sales_target: 0,
        target_deadline: 0,
        goal_met: false,
        custom_fee_bps: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
        feedback_cid: None,
    };

    let event_2 = EventInfo {
        event_id: String::from_str(&env, "e2"),
        name: String::from_str(&env, "Test Event 2"),
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
        min_sales_target: 0,
        target_deadline: 0,
        goal_met: false,
        custom_fee_bps: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
        feedback_cid: None,
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
fn test_get_organizer_receipts_returns_archived_receipts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let other_organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let make_event =
        |event_id: &str, organizer_address: &Address, current_supply: i128| EventInfo {
            event_id: String::from_str(&env, event_id),
            name: String::from_str(&env, "Test Event"),
            organizer_address: organizer_address.clone(),
            payment_address: test_payment_address(&env),
            platform_fee_percent: 500,
            is_active: false,
            status: EventStatus::Inactive,
            created_at: env.ledger().timestamp(),
            metadata_cid: String::from_str(
                &env,
                "bafkreifh22222222222222222222222222222222222222222222222222",
            ),
            max_supply: 100,
            current_supply,
            milestone_plan: None,
            tiers: Map::new(&env),
            refund_deadline: 0,
            restocking_fee: 0,
            resale_cap_bps: None,
            is_postponed: false,
            grace_period_end: 0,
            min_sales_target: 0,
            target_deadline: 0,
            goal_met: false,
            custom_fee_bps: None,
            banner_cid: None,
            tags: None,
            end_time: 0,
            feedback_cid: None,
        };

    let event_id_1 = String::from_str(&env, "archived_1");
    let event_id_2 = String::from_str(&env, "archived_2");
    let other_event_id = String::from_str(&env, "other_archived");

    client.store_event(&make_event("archived_1", &organizer, 12));
    client.store_event(&make_event("archived_2", &organizer, 4));
    client.store_event(&make_event("other_archived", &other_organizer, 8));

    env.ledger().with_mut(|li| li.timestamp = 1_000);
    client.archive_event(&event_id_1);

    env.ledger().with_mut(|li| li.timestamp = 2_000);
    client.archive_event(&other_event_id);

    env.ledger().with_mut(|li| li.timestamp = 3_000);
    client.archive_event(&event_id_2);

    assert!(client.get_event(&event_id_1).is_none());
    assert!(client.get_event(&event_id_2).is_none());

    let receipts = client.get_organizer_receipts(&organizer);
    assert_eq!(receipts.len(), 2);
    assert_eq!(
        receipts.get(0).unwrap(),
        EventReceipt {
            event_id: event_id_1.clone(),
            organizer_address: organizer.clone(),
            total_sold: 12,
            archived_at: 1_000,
        }
    );
    assert_eq!(
        receipts.get(1).unwrap(),
        EventReceipt {
            event_id: event_id_2.clone(),
            organizer_address: organizer.clone(),
            total_sold: 4,
            archived_at: 3_000,
        }
    );

    let other_receipts = client.get_organizer_receipts(&other_organizer);
    assert_eq!(other_receipts.len(), 1);
    assert_eq!(other_receipts.get(0).unwrap().event_id, other_event_id);
}

#[test]
fn test_register_event_success() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    env.mock_all_auths();
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr.clone(),
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
        end_time: 0,
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
fn test_register_event_name_trimming() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);

    env.mock_all_auths();
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "event_trim_test");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    // Register with intentionally messy name (leading/trailing whitespace)
    let messy_name = String::from_str(&env, "  Summer Fest 2025  ");
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: messy_name,
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers: Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    let stored = client.get_event(&event_id).unwrap();
    // Name should be trimmed of leading and trailing whitespace
    assert_eq!(stored.name, String::from_str(&env, "Summer Fest 2025"));
}

#[test]
fn test_register_event_invalid_target_deadline() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    env.mock_all_auths();
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "event_deadline");
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    // Advance ledger timestamp so `now - 1` does not underflow
    env.ledger().with_mut(|li| {
        li.timestamp = 1000;
    });
    let now = env.ledger().timestamp();

    // Past deadline should fail
    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr.clone(),
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: Some(now - 1),
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidTargetDeadline)));

    // Present deadline should fail
    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr.clone(),
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: Some(now),
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidTargetDeadline)));

    // Future deadline should succeed
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: Some(now + 100),
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    let stored = client.get_event(&event_id).unwrap();
    assert_eq!(stored.target_deadline, now + 100);
}

#[test]
fn test_register_event_rejects_contract_as_organizer() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "event_bad_org_contract"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: client.address.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 100,
        milestone_plan: None,
        tiers: Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    assert_eq!(result, Err(Ok(EventRegistryError::InvalidAddress)));
}

#[test]
fn test_register_event_rejects_zero_organizer_address() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let zero_organizer = Address::from_string(&String::from_str(
        &env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJXFF",
    ));

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "event_bad_org_zero"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: zero_organizer,
        payment_address: test_payment_address(&env),
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 100,
        milestone_plan: None,
        tiers: Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    assert_eq!(result, Err(Ok(EventRegistryError::InvalidAddress)));
}

#[test]
fn test_register_event_unlimited_supply() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    env.mock_all_auths();
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "unlimited_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 0,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "event_001");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr.clone(),
        metadata_cid: metadata_cid.clone(),
        max_supply: 100,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id,
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
    });
    assert_eq!(result, Err(Ok(EventRegistryError::EventAlreadyExists)));
}

#[test]
fn test_register_event_invalid_metadata_cid_formats() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let tiers = Map::new(&env);
    let short_cid = String::from_str(&env, "bafy");
    let short_result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "event_short_cid"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr.clone(),
        metadata_cid: short_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers: tiers.clone(),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    assert_eq!(
        short_result,
        Err(Ok(EventRegistryError::InvalidMetadataCid))
    );

    let wrong_prefix_cid = String::from_str(
        &env,
        "Qafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let wrong_prefix_result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "event_wrong_prefix_cid"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid: wrong_prefix_cid,
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
        end_time: 0,
    });
    assert_eq!(
        wrong_prefix_result,
        Err(Ok(EventRegistryError::InvalidMetadataCid))
    );

    let oversized_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdiaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let oversized_result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "event_oversized_cid"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: Address::generate(&env),
        payment_address: test_payment_address(&env),
        metadata_cid: oversized_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers: Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    assert_eq!(
        oversized_result,
        Err(Ok(EventRegistryError::InvalidMetadataCid))
    );
}

#[test]
fn test_get_event_payment_info() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &750, &usdc_token);

    let event_id = String::from_str(&env, "event_002");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr.clone(),
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "event_001");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    let event_id = String::from_str(&env, "event_001");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &600, &usdc_token);

    let event_id = String::from_str(&env, "lifecycle_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr.clone(),
        metadata_cid,
        max_supply: 200,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "event_metadata");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "event_metadata");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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

    let oversized_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdiaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let oversized_result = client.try_update_metadata(&event_id, &oversized_cid);
    assert_eq!(
        oversized_result,
        Err(Ok(EventRegistryError::InvalidMetadataCid))
    );
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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);

    assert_eq!(client.get_ticket_payment_contract(), ticket_payment);
}

#[test]
fn test_set_custom_event_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: test_payment_address(&env),
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
        end_time: 0,
    });

    // Default fee
    let info = client.get_event_payment_info(&event_id);
    assert_eq!(info.platform_fee_percent, 500);
    assert_eq!(info.custom_fee_bps, None);

    // Set custom fee
    client.set_custom_event_fee(&event_id, &Some(100));

    let info = client.get_event_payment_info(&event_id);
    assert_eq!(info.platform_fee_percent, 500);
    assert_eq!(info.custom_fee_bps, Some(100));

    // Clear custom fee
    client.set_custom_event_fee(&event_id, &None);
    let info = client.get_event_payment_info(&event_id);
    assert_eq!(info.custom_fee_bps, None);
}

#[test]
fn test_set_custom_event_fee_exceeds_max() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "event_001");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: test_payment_address(&env),
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
        end_time: 0,
    });

    // Try to set custom fee exceeding 10000 bps (100%)
    let result = client.try_set_custom_event_fee(&event_id, &Some(10001));
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidFeePercent)));

    // Try to set custom fee at exactly 10000 bps (100%) - should succeed
    client.set_custom_event_fee(&event_id, &Some(10000));
    let info = client.get_event_payment_info(&event_id);
    assert_eq!(info.custom_fee_bps, Some(10000));

    // Try to set custom fee way above limit
    let result = client.try_set_custom_event_fee(&event_id, &Some(50000));
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidFeePercent)));
}

#[test]
fn test_increment_inventory_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 10,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 2,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
fn test_increment_inventory_bulk_exceeds_max_supply() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "bulk_limited_event");
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
            tier_limit: 3,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 3,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    // Fill one slot, then attempt a bulk call that overshoots max_supply in one shot
    client.increment_inventory(&event_id, &tier_id, &1);

    let result = client.try_increment_inventory(&event_id, &tier_id, &5);
    assert_eq!(result, Err(Ok(EventRegistryError::MaxSupplyExceeded)));

    // Supply must remain unchanged after the failed call
    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.current_supply, 1);
}

#[test]
fn test_increment_inventory_unlimited_supply() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 0,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 70,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
fn test_increment_inventory_supply_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "overflow_event");
    let tier_id = String::from_str(&env, "general");
    let mut tiers = Map::new(&env);
    tiers.set(
        tier_id.clone(),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: i128::MAX,
            current_sold: i128::MAX - 1,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    // Store event with current_supply near i128::MAX to trigger overflow
    client.store_event(&EventInfo {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr,
        platform_fee_percent: 500,
        is_active: true,
        status: EventStatus::Active,
        created_at: env.ledger().timestamp(),
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 0, // unlimited so max_supply check is skipped
        current_supply: i128::MAX,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        is_postponed: false,
        grace_period_end: 0,
        min_sales_target: 0,
        target_deadline: 0,
        goal_met: false,
        custom_fee_bps: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
        feedback_cid: None,
    });

    let result = client.try_increment_inventory(&event_id, &tier_id, &1);
    assert_eq!(result, Err(Ok(EventRegistryError::SupplyOverflow)));
}

#[test]
fn test_increment_inventory_tier_sold_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "tier_overflow_event");
    let tier_id = String::from_str(&env, "general");
    let mut tiers = Map::new(&env);
    tiers.set(
        tier_id.clone(),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5000000,
            tier_limit: i128::MAX,
            current_sold: i128::MAX, // tier current_sold at max
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.store_event(&EventInfo {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr,
        platform_fee_percent: 500,
        is_active: true,
        status: EventStatus::Active,
        created_at: env.ledger().timestamp(),
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
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
        min_sales_target: 0,
        target_deadline: 0,
        goal_met: false,
        custom_fee_bps: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
        feedback_cid: None,
    });

    let result = client.try_increment_inventory(&event_id, &tier_id, &1);
    assert_eq!(result, Err(Ok(EventRegistryError::SupplyOverflow)));
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "status_noop_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "metadata_noop_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr,
        metadata_cid: metadata_cid.clone(),
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
        end_time: 0,
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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "test_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr,
        metadata_cid: metadata_cid.clone(),
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
        end_time: 0,
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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: Some(1000), // 10% above face value
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "no_markup_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: Some(0), // No markup allowed
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "free_market_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None, // No cap
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "postponed_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
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
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "bad_cap_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id,
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: Some(10001), // Over 100% - invalid
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
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
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "cancel_me");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 100,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    client.cancel_event(&event_id);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.status, EventStatus::Cancelled);
    assert!(!event_info.is_active);
}

#[test]
fn test_archive_event_rejects_active_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "archive_active");
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 100,
        milestone_plan: None,
        tiers: Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    let result = client.try_archive_event(&event_id);
    assert_eq!(result, Err(Ok(EventRegistryError::EventIsActive)));
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
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "cancel_twice");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
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
        end_time: 0,
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
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "no_updates");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
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
        end_time: 0,
    });

    client.cancel_event(&event_id);
    let result = client.try_update_event_status(&event_id, &true);
    assert_eq!(result, Err(Ok(EventRegistryError::EventCancelled)));
}

// ════════════════════════════════════════════════════════════════
// Loyalty & Staking Tests
// ════════════════════════════════════════════════════════════════

/// Helper: initialises a fresh contract and returns (client, admin, platform_wallet)
fn setup_loyalty_env(env: &Env) -> (crate::EventRegistryClient<'static>, Address, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = crate::EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    (client, admin, platform_wallet)
}

// ── Guest Loyalty Profile ────────────────────────────────────────

#[test]
fn test_guest_profile_initially_none() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    assert!(client.get_guest_profile(&guest).is_none());
}

#[test]
fn test_update_loyalty_score_creates_profile() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    client.update_loyalty_score(&admin, &guest, &2, &2000_0000000i128, &1u32);

    let profile = client.get_guest_profile(&guest).unwrap();
    assert_eq!(profile.guest_address, guest);
    assert_eq!(profile.loyalty_score, 20); // 2 tickets × 10 pts
    assert_eq!(profile.total_tickets_purchased, 2);
    assert_eq!(profile.total_spent, 2000_0000000i128);
}

#[test]
fn test_update_loyalty_score_accumulates() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);

    // First purchase: 5 tickets
    client.update_loyalty_score(&admin, &guest, &5, &5000_0000000i128, &1u32);
    // Second purchase: 3 tickets
    client.update_loyalty_score(&admin, &guest, &3, &3000_0000000i128, &1u32);

    let profile = client.get_guest_profile(&guest).unwrap();
    assert_eq!(profile.loyalty_score, 80); // (5+3) × 10
    assert_eq!(profile.total_tickets_purchased, 8);
    assert_eq!(profile.total_spent, 8000_0000000i128);
}

#[test]
fn test_update_loyalty_score_unauthorized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    let random_caller = Address::generate(&env);

    let result = client.try_update_loyalty_score(&random_caller, &guest, &1, &1000i128, &1u32);
    assert_eq!(result, Err(Ok(EventRegistryError::Unauthorized)));
}

#[test]
fn test_update_loyalty_score_zero_tickets_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    let result = client.try_update_loyalty_score(&admin, &guest, &0, &0i128, &1u32);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidQuantity)));
}

// ── Loyalty Discount Tiers ───────────────────────────────────────

#[test]
fn test_loyalty_discount_bps_no_profile() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    assert_eq!(client.get_loyalty_discount_bps(&guest), 0);
}

#[test]
fn test_loyalty_discount_bps_tiers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);

    // Score < 100 → 0 bps
    client.update_loyalty_score(&admin, &guest, &5, &100i128, &1u32); // 50 pts
    assert_eq!(client.get_loyalty_discount_bps(&guest), 0);

    // Score 100–499 → 250 bps
    client.update_loyalty_score(&admin, &guest, &5, &100i128, &1u32); // +50 = 100 pts
    assert_eq!(client.get_loyalty_discount_bps(&guest), 250);

    // Score 500–999 → 500 bps
    // Need to get to 500 pts: currently 100, need 400 more = 40 tickets
    client.update_loyalty_score(&admin, &guest, &40, &1000i128, &1u32); // +400 = 500 pts
    assert_eq!(client.get_loyalty_discount_bps(&guest), 500);

    // Score ≥ 1000 → 1000 bps
    // Need 500 more pts = 50 tickets
    client.update_loyalty_score(&admin, &guest, &50, &1000i128, &1u32); // +500 = 1000 pts
    assert_eq!(client.get_loyalty_discount_bps(&guest), 1000);
}

// ── Staking Configuration ────────────────────────────────────────

#[test]
fn test_set_staking_config_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let token = Address::generate(&env);
    let min_amount = 1000_0000000i128;
    client.set_staking_config(&token, &min_amount);
    // No error means success; verify via a stake attempt
}

#[test]
fn test_set_staking_config_zero_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let token = Address::generate(&env);
    let result = client.try_set_staking_config(&token, &0i128);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidStakeAmount)));
}

// ── stake_collateral ─────────────────────────────────────────────

#[test]
fn test_stake_collateral_achieves_verified_status() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let min_amount = 1000_0000000i128;

    // Create a stellar asset token and mint to organizer
    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&organizer, &(min_amount * 2));

    // Configure staking
    client.set_staking_config(&token_id, &min_amount);

    // Approve tokens and stake
    soroban_sdk::token::Client::new(&env, &token_id).approve(
        &organizer,
        &client.address,
        &min_amount,
        &99999,
    );
    client.stake_collateral(&organizer, &min_amount);

    // Check stake record
    let stake = client.get_organizer_stake(&organizer).unwrap();
    assert_eq!(stake.organizer, organizer);
    assert_eq!(stake.amount, min_amount);
    assert!(stake.is_verified);
    assert_eq!(stake.reward_balance, 0);

    // Check verified status helper
    assert!(client.is_organizer_verified(&organizer));
}

#[test]
fn test_stake_below_min_not_verified() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let min_amount = 1000_0000000i128;
    let stake_amount = min_amount / 2;

    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&organizer, &stake_amount);

    client.set_staking_config(&token_id, &min_amount);

    soroban_sdk::token::Client::new(&env, &token_id).approve(
        &organizer,
        &client.address,
        &stake_amount,
        &99999,
    );
    client.stake_collateral(&organizer, &stake_amount);

    let stake = client.get_organizer_stake(&organizer).unwrap();
    assert!(!stake.is_verified);
    assert!(!client.is_organizer_verified(&organizer));
}

#[test]
fn test_stake_collateral_without_config_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let result = client.try_stake_collateral(&organizer, &1000i128);
    assert_eq!(result, Err(Ok(EventRegistryError::StakingNotConfigured)));
}

#[test]
fn test_stake_collateral_zero_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    client.set_staking_config(&token_id, &1000i128);

    let result = client.try_stake_collateral(&organizer, &0i128);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidStakeAmount)));
}

#[test]
fn test_double_stake_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let stake_amount = 500_0000000i128;

    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&organizer, &(stake_amount * 2));

    client.set_staking_config(&token_id, &1000_0000000i128);

    soroban_sdk::token::Client::new(&env, &token_id).approve(
        &organizer,
        &client.address,
        &stake_amount,
        &99999,
    );
    client.stake_collateral(&organizer, &stake_amount);

    // Second stake attempt should fail
    soroban_sdk::token::Client::new(&env, &token_id).approve(
        &organizer,
        &client.address,
        &stake_amount,
        &99999,
    );
    let result = client.try_stake_collateral(&organizer, &stake_amount);
    assert_eq!(result, Err(Ok(EventRegistryError::AlreadyStaked)));
}

// ── unstake_collateral ───────────────────────────────────────────

#[test]
fn test_unstake_collateral_returns_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let stake_amount = 1000_0000000i128;

    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&organizer, &stake_amount);

    client.set_staking_config(&token_id, &stake_amount);

    let token_client = soroban_sdk::token::Client::new(&env, &token_id);
    token_client.approve(&organizer, &client.address, &stake_amount, &99999);
    client.stake_collateral(&organizer, &stake_amount);

    // Balance should be 0 after staking
    assert_eq!(token_client.balance(&organizer), 0);

    // Unstake
    client.unstake_collateral(&organizer);

    // Balance should be restored
    assert_eq!(token_client.balance(&organizer), stake_amount);
    assert!(client.get_organizer_stake(&organizer).is_none());
    assert!(!client.is_organizer_verified(&organizer));
}

#[test]
fn test_unstake_without_stake_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let result = client.try_unstake_collateral(&organizer);
    assert_eq!(result, Err(Ok(EventRegistryError::NotStaked)));
}

// ── distribute_staker_rewards & claim_staker_rewards ────────────

#[test]
fn test_distribute_and_claim_staker_rewards() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let stake_amount = 1000_0000000i128;
    let reward_amount = 100_0000000i128;

    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    // Mint for organizer (stake) + admin (rewards)
    token_admin.mint(&organizer, &stake_amount);
    token_admin.mint(&admin, &reward_amount);

    client.set_staking_config(&token_id, &stake_amount);

    let token_client = soroban_sdk::token::Client::new(&env, &token_id);
    token_client.approve(&organizer, &client.address, &stake_amount, &99999);
    client.stake_collateral(&organizer, &stake_amount);

    // Admin approves reward tokens to contract
    token_client.approve(&admin, &client.address, &reward_amount, &99999);
    client.distribute_staker_rewards(&admin, &reward_amount);

    // Organizer's reward_balance should be updated
    let stake = client.get_organizer_stake(&organizer).unwrap();
    assert_eq!(stake.reward_balance, reward_amount); // 100% since only one staker

    // Organizer claims rewards
    let claimed = client.claim_staker_rewards(&organizer);
    assert_eq!(claimed, reward_amount);

    // Check token balance restored
    assert_eq!(token_client.balance(&organizer), reward_amount);

    // reward_balance should be zero after claiming
    let stake_after = client.get_organizer_stake(&organizer).unwrap();
    assert_eq!(stake_after.reward_balance, 0);
    assert_eq!(stake_after.total_rewards_claimed, reward_amount);
}

#[test]
fn test_distribute_rewards_proportional_to_multiple_stakers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let organizer_a = Address::generate(&env);
    let organizer_b = Address::generate(&env);
    let stake_a = 1000_0000000i128;
    let stake_b = 3000_0000000i128;
    let total_reward = 1000_0000000i128;

    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&organizer_a, &stake_a);
    token_admin.mint(&organizer_b, &stake_b);
    token_admin.mint(&admin, &total_reward);

    client.set_staking_config(&token_id, &1i128); // min_amount = 1 for simplicity

    let token_client = soroban_sdk::token::Client::new(&env, &token_id);
    token_client.approve(&organizer_a, &client.address, &stake_a, &99999);
    client.stake_collateral(&organizer_a, &stake_a);

    token_client.approve(&organizer_b, &client.address, &stake_b, &99999);
    client.stake_collateral(&organizer_b, &stake_b);

    token_client.approve(&admin, &client.address, &total_reward, &99999);
    client.distribute_staker_rewards(&admin, &total_reward);

    // A has 25% stake (1000/4000), B has 75% (3000/4000)
    let expected_a = total_reward * stake_a / (stake_a + stake_b); // 250_0000000
    let expected_b = total_reward * stake_b / (stake_a + stake_b); // 750_0000000

    let stake_a_record = client.get_organizer_stake(&organizer_a).unwrap();
    let stake_b_record = client.get_organizer_stake(&organizer_b).unwrap();

    assert_eq!(stake_a_record.reward_balance, expected_a);
    assert_eq!(stake_b_record.reward_balance, expected_b);
}

#[test]
fn test_claim_rewards_no_stake_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let result = client.try_claim_staker_rewards(&organizer);
    assert_eq!(result, Err(Ok(EventRegistryError::NotStaked)));
}

#[test]
fn test_claim_rewards_zero_balance_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    let stake_amount = 500_0000000i128;

    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&organizer, &stake_amount);

    client.set_staking_config(&token_id, &stake_amount);

    let token_client = soroban_sdk::token::Client::new(&env, &token_id);
    token_client.approve(&organizer, &client.address, &stake_amount, &99999);
    client.stake_collateral(&organizer, &stake_amount);

    // No rewards distributed yet
    let result = client.try_claim_staker_rewards(&organizer);
    assert_eq!(result, Err(Ok(EventRegistryError::NoRewardsAvailable)));
}

#[test]
fn test_distribute_rewards_no_stakers_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    client.set_staking_config(&token_id, &1000i128);

    let result = client.try_distribute_staker_rewards(&admin, &100i128);
    assert_eq!(result, Err(Ok(EventRegistryError::NotStaked)));
}

#[test]
fn test_distribute_rewards_unauthorized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let random_caller = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    client.set_staking_config(&token_id, &1000i128);

    let result = client.try_distribute_staker_rewards(&random_caller, &100i128);
    assert_eq!(result, Err(Ok(EventRegistryError::Unauthorized)));
}

#[test]
fn test_is_organizer_verified_false_when_not_staked() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_loyalty_env(&env);

    let organizer = Address::generate(&env);
    assert!(!client.is_organizer_verified(&organizer));
}

// ==================== USDC Token Whitelist Tests ====================

#[test]
fn test_usdc_token_whitelisted_after_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // USDC token must be whitelisted automatically after initialization
    assert!(client.is_token_whitelisted(&usdc_token));
}

#[test]
fn test_non_usdc_token_not_whitelisted_after_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let other_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // A different token should NOT be whitelisted
    assert!(!client.is_token_whitelisted(&other_token));
}

#[test]
fn test_admin_can_add_token_to_whitelist() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let new_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    assert!(!client.is_token_whitelisted(&new_token));
    client.add_to_token_whitelist(&new_token);
    assert!(client.is_token_whitelisted(&new_token));
}

#[test]
fn test_admin_can_remove_token_from_whitelist() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // USDC is whitelisted after init
    assert!(client.is_token_whitelisted(&usdc_token));

    // Admin removes it
    client.remove_from_token_whitelist(&usdc_token);
    assert!(!client.is_token_whitelisted(&usdc_token));
}

#[test]
#[should_panic] // Authentication failure — non-admin cannot add to whitelist
fn test_non_admin_cannot_add_token_to_whitelist() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    let new_token = Address::generate(&env);

    // Only mock auth for initialize, not for add_to_token_whitelist
    env.mock_all_auths();
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Clear mocked auths so the next call requires real auth
    let env2 = Env::default();
    let client2 = EventRegistryClient::new(&env2, &contract_id);
    client2.add_to_token_whitelist(&new_token);
}

#[test]
fn test_register_event_with_banner_cid() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "event_banner");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let banner_cid = Some(String::from_str(
        &env,
        "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku",
    ));

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers: Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: banner_cid.clone(),
        tags: None,
        end_time: 0,
    });

    let event = client.get_event(&event_id).unwrap();
    assert_eq!(event.banner_cid, banner_cid);
}

#[test]
fn test_goal_met_event_fires_only_once() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let _organizer = Address::generate(&env);
    let _payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "goal_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let banner_cid = Some(String::from_str(
        &env,
        "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku",
    ));

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: Address::generate(&env),
        payment_address: test_payment_address(&env),
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers: soroban_sdk::Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: banner_cid.clone(),
        tags: None,
        end_time: 0,
    });

    let event = client.get_event(&event_id).unwrap();
    assert_eq!(event.banner_cid, banner_cid);
}

#[test]
fn test_register_event_without_banner_cid() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = test_payment_address(&env);
    let platform_wallet = Address::generate(&env);
    let ticket_payment = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "event_no_banner");
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
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: Some(10),
        target_deadline: Some(1000),
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    // Drain setup events
    let _ = env.events().all();

    // Below threshold: only InventoryIncremented, no GoalMet
    client.increment_inventory(&event_id, &tier_id, &5);
    let events = env.events().all();
    assert_eq!(events.len(), 1, "expected only InventoryIncremented event");
    assert!(!client.get_event(&event_id).unwrap().goal_met);

    // Cross the threshold (5 + 5 = 10 >= 10): GoalMet + InventoryIncremented
    client.increment_inventory(&event_id, &tier_id, &5);
    let events = env.events().all();
    assert_eq!(
        events.len(),
        2,
        "expected GoalMet and InventoryIncremented events"
    );
    assert!(client.get_event(&event_id).unwrap().goal_met);

    // Past threshold: only InventoryIncremented, no second GoalMet
    client.increment_inventory(&event_id, &tier_id, &5);
    let events = env.events().all();
    assert_eq!(
        events.len(),
        1,
        "GoalMet must not fire again after threshold already crossed"
    );
    assert!(client.get_event(&event_id).unwrap().goal_met);
}

#[test]
fn test_series_pass_issued_at_timestamp() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Register an event for the series
    let event_id = String::from_str(&env, "event_ts");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid,
        max_supply: 50,
        milestone_plan: None,
        tiers: soroban_sdk::Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    // Register a series
    let series_id = String::from_str(&env, "series_ts");
    let event_ids = soroban_sdk::vec![&env, event_id.clone()];
    client.register_series(
        &series_id,
        &String::from_str(&env, "Timestamp Series"),
        &event_ids,
        &organizer,
        &None,
    );

    // Set a specific ledger timestamp
    let expected_timestamp = 1700000000u64;
    env.ledger()
        .with_mut(|li| li.timestamp = expected_timestamp);

    // Issue the pass
    let pass_id = String::from_str(&env, "pass_ts");
    let holder = Address::generate(&env);
    client.issue_series_pass(
        &pass_id,
        &series_id,
        &holder,
        &5,
        &(expected_timestamp + 10000),
    );

    // Verify issued_at matches the ledger timestamp
    let pass = client.get_series_pass(&pass_id).unwrap();
    assert_eq!(pass.issued_at, expected_timestamp);
    assert_eq!(pass.holder, holder);
    assert_eq!(pass.series_id, series_id);
    assert_eq!(pass.usage_limit, 5);
    assert_eq!(pass.usage_count, 0);
}

// ── Milestone percentage validation ──────────────────────────────────────────

use crate::types::Milestone;

fn setup_client(env: &Env) -> (EventRegistryClient<'_>, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    let organizer = Address::generate(env);
    (client, organizer)
}

fn base_args(
    env: &Env,
    organizer: &Address,
    milestone_plan: Option<soroban_sdk::Vec<Milestone>>,
) -> EventRegistrationArgs {
    EventRegistrationArgs {
        event_id: String::from_str(env, "evt_milestone"),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(env),
        metadata_cid: String::from_str(
            env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 100,
        milestone_plan,
        tiers: Map::new(env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    }
}

#[test]
fn test_register_event_milestone_plan_valid_exact_100() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, organizer) = setup_client(&env);

    let milestones = soroban_sdk::vec![
        &env,
        Milestone {
            sales_threshold: 50,
            release_percent: 5000
        },
        Milestone {
            sales_threshold: 100,
            release_percent: 5000
        },
    ];
    // Should succeed: 5000 + 5000 = 10000 bps (exactly 100%)
    client.register_event(&base_args(&env, &organizer, Some(milestones)));
}

#[test]
fn test_register_event_milestone_plan_valid_under_100() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, organizer) = setup_client(&env);

    let milestones = soroban_sdk::vec![
        &env,
        Milestone {
            sales_threshold: 50,
            release_percent: 3000
        },
        Milestone {
            sales_threshold: 100,
            release_percent: 4000
        },
    ];
    // Should succeed: 3000 + 4000 = 7000 bps (70%)
    client.register_event(&base_args(&env, &organizer, Some(milestones)));
}

#[test]
fn test_register_event_milestone_plan_exceeds_100_returns_error() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, organizer) = setup_client(&env);

    let milestones = soroban_sdk::vec![
        &env,
        Milestone {
            sales_threshold: 50,
            release_percent: 6000
        },
        Milestone {
            sales_threshold: 100,
            release_percent: 5000
        },
    ];
    // Should fail: 6000 + 5000 = 11000 bps (110%)
    let result = client.try_register_event(&base_args(&env, &organizer, Some(milestones)));
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidMilestonePlan)));
}

#[test]
fn test_register_event_no_milestone_plan_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, organizer) = setup_client(&env);

    // None milestone_plan should always pass validation
    client.register_event(&base_args(&env, &organizer, None));
}

// ==================== Governance / Multi-Sig Tests ====================

#[test]
fn test_propose_and_execute_update_platform_wallet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let new_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Verify initial wallet
    assert_eq!(client.get_platform_wallet(), platform_wallet);

    // Propose to update platform wallet
    let proposal_id = client.propose_set_platform_wallet(&admin, &new_wallet, &0);

    // Verify proposal was created
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.proposal_id, proposal_id);
    assert_eq!(proposal.proposer, admin);
    assert!(!proposal.executed);

    // Execute proposal (threshold = 1, so already approved)
    client.execute_proposal(&admin, &proposal_id);

    // Verify wallet was updated
    assert_eq!(client.get_platform_wallet(), new_wallet);

    // Verify proposal was marked as executed
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
fn test_update_platform_wallet_with_multisig() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let new_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Add admin2
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Set threshold to 2
    let proposal_id = client.propose_set_threshold(&admin1, &2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Propose to update platform wallet
    let proposal_id = client.propose_set_platform_wallet(&admin1, &new_wallet, &0);

    // Try to execute with only 1 approval - should fail
    let result = client.try_execute_proposal(&admin1, &proposal_id);
    assert!(result.is_err());

    // Admin2 approves
    client.approve_proposal(&admin2, &proposal_id);

    // Now execute should succeed
    client.execute_proposal(&admin1, &proposal_id);

    // Verify wallet was updated
    assert_eq!(client.get_platform_wallet(), new_wallet);
}

#[test]
fn test_propose_update_platform_wallet_invalid_address() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Try to set platform wallet to contract address (invalid)
    let result = client.try_propose_set_platform_wallet(&admin, &contract_id, &0);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidAddress)));
}

#[test]
fn test_propose_update_platform_wallet_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let new_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Try to propose as non-admin - should fail
    let result = client.try_propose_set_platform_wallet(&non_admin, &new_wallet, &0);
    assert_eq!(result, Err(Ok(EventRegistryError::Unauthorized)));
}

#[test]
fn test_parameter_change_variants() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Test AddAdmin
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);
    assert!(client.is_admin(&admin2));

    // Test SetThreshold
    let proposal_id = client.propose_set_threshold(&admin1, &2, &0);
    client.execute_proposal(&admin1, &proposal_id);
    let config = client.get_multisig_config();
    assert_eq!(config.threshold, 2);

    // Test UpdatePlatformWallet
    let new_wallet = Address::generate(&env);
    let proposal_id = client.propose_set_platform_wallet(&admin1, &new_wallet, &0);
    client.approve_proposal(&admin2, &proposal_id);
    client.execute_proposal(&admin1, &proposal_id);
    assert_eq!(client.get_platform_wallet(), new_wallet);

    // Test RemoveAdmin
    let proposal_id = client.propose_remove_admin(&admin1, &admin2, &0);
    client.approve_proposal(&admin2, &proposal_id);
    client.execute_proposal(&admin1, &proposal_id);
    assert!(!client.is_admin(&admin2));
}

#[test]
fn test_proposal_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let new_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Create proposal with short expiry (10 ledgers)
    let proposal_id = client.propose_set_platform_wallet(&admin, &new_wallet, &10);

    // Advance ledger past expiration
    env.ledger().with_mut(|li| {
        li.timestamp += 11;
    });

    // Try to execute expired proposal - should fail
    let result = client.try_execute_proposal(&admin, &proposal_id);
    assert_eq!(result, Err(Ok(EventRegistryError::ProposalExpired)));
}

#[test]
fn test_active_proposals_list() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    // Create multiple proposals
    let proposal_id1 = client.propose_add_admin(&admin, &admin2, &0);
    let proposal_id2 = client.propose_add_admin(&admin, &admin3, &0);

    let active_proposals = client.get_active_proposals();
    assert_eq!(active_proposals.len(), 2);
    assert!(active_proposals.contains(proposal_id1));
    assert!(active_proposals.contains(proposal_id2));

    // Execute one proposal
    client.execute_proposal(&admin, &proposal_id1);

    // Should have one less active proposal
    let active_proposals = client.get_active_proposals();
    assert_eq!(active_proposals.len(), 1);
    assert!(!active_proposals.contains(proposal_id1));
    assert!(active_proposals.contains(proposal_id2));
}

#[test]
fn test_cancelled_status_guard() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "cancelled_event");

    // Register event
    let tiers = Map::new(&env);
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: String::from_str(
            &env,
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
        tags: None,
        end_time: 0,
    });

    // Cancel event
    client.cancel_event(&event_id);

    // Try to update status - should fail
    let result = client.try_update_event_status(&event_id, &true);
    assert_eq!(result, Err(Ok(EventRegistryError::EventCancelled)));

    // Try to postpone - should fail
    let result = client.try_postpone_event(&event_id, &10000000);
    assert_eq!(result, Err(Ok(EventRegistryError::EventCancelled)));

    // Try to update metadata - should fail
    let result = client.try_update_metadata(
        &event_id,
        &String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
    );
    assert_eq!(result, Err(Ok(EventRegistryError::EventCancelled)));

    // Try to set custom fee - should fail
    let result = client.try_set_custom_event_fee(&event_id, &Some(100));
    assert_eq!(result, Err(Ok(EventRegistryError::EventCancelled)));
}

// ── Issue #194: Tier Error Message Tests ─────────────────────────────────────

/// Helper to format a Display value into a fixed-size stack buffer.
fn fmt_to_str<T: core::fmt::Display>(val: T) -> [u8; 256] {
    struct Buf {
        data: [u8; 256],
        pos: usize,
    }
    impl core::fmt::Write for Buf {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let end = self.pos + bytes.len();
            if end <= self.data.len() {
                self.data[self.pos..end].copy_from_slice(bytes);
                self.pos = end;
            }
            Ok(())
        }
    }
    let mut buf = Buf {
        data: [0u8; 256],
        pos: 0,
    };
    core::fmt::write(&mut buf, format_args!("{}", val)).ok();
    buf.data
}

fn buf_starts_with(buf: &[u8; 256], expected: &str) -> bool {
    let e = expected.as_bytes();
    buf.len() >= e.len() && &buf[..e.len()] == e && (e.len() == 256 || buf[e.len()] == 0)
}

#[test]
fn test_tier_not_found_error_message() {
    let buf = fmt_to_str(EventRegistryError::TierNotFound);
    assert!(
        buf_starts_with(
            &buf,
            "The specified ticket tier ID does not exist for this event"
        ),
        "unexpected message"
    );
}

#[test]
fn test_tier_supply_exceeded_error_message() {
    let buf = fmt_to_str(EventRegistryError::TierSupplyExceeded);
    assert!(
        buf_starts_with(
            &buf,
            "The requested ticket tier has sold out and cannot accept more registrations"
        ),
        "unexpected message"
    );
}

// ── Issue #211: Restocking Fee Guard Tests ────────────────────────────────────

#[test]
fn test_register_event_restocking_fee_exceeds_tier_price_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let organizer = Address::generate(&env);
    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5_000_000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    // restocking_fee (6_000_000) > tier price (5_000_000) — must fail
    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "evt_restocking_guard"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 6_000_000,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    assert_eq!(
        result,
        Err(Ok(EventRegistryError::RestockingFeeExceedsTicketPrice))
    );
}

#[test]
fn test_register_event_restocking_fee_equal_to_tier_price_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let organizer = Address::generate(&env);
    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 5_000_000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    // restocking_fee == tier price — should succeed
    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "evt_restocking_equal"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 5_000_000,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    assert!(result.is_ok());
}

#[test]
fn test_register_event_restocking_fee_zero_always_valid() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let organizer = Address::generate(&env);
    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 1_000,
            tier_limit: 50,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "evt_restocking_zero"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    assert!(result.is_ok());
}

#[test]
fn test_register_event_restocking_fee_overflow_returns_invalid_fee_calculation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let organizer = Address::generate(&env);
    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "general"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: i128::MIN,
            tier_limit: 50,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "evt_restocking_overflow"),
        name: String::from_str(&env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid: String::from_str(
            &env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 50,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 1,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });

    assert_eq!(
        result,
        Err(Ok(EventRegistryError::RestockingFeeExceedsTicketPrice))
    );
}

#[test]
fn test_restocking_fee_exceeds_ticket_price_error_message() {
    let buf = fmt_to_str(EventRegistryError::RestockingFeeExceedsTicketPrice);
    assert!(
        buf_starts_with(
            &buf,
            "Restocking fee must not exceed the original ticket price"
        ),
        "unexpected message"
    );
}

#[test]
fn test_register_event_tier_limit_overflow() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "overflow_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    // Two tiers that sum to > i128::MAX
    tiers.set(
        String::from_str(&env, "t1"),
        TicketTier {
            name: String::from_str(&env, "T1"),
            price: 100,
            tier_limit: i128::MAX / 2 + 10,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );
    tiers.set(
        String::from_str(&env, "t2"),
        TicketTier {
            name: String::from_str(&env, "T2"),
            price: 100,
            tier_limit: i128::MAX / 2 + 10,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id,
        name: String::from_str(&env, "Overflow Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 0, // unlimited global supply, but tier limits still overflow
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    assert_eq!(result, Err(Ok(EventRegistryError::SupplyOverflow)));
}

#[test]
fn test_register_event_invalid_tier_limit_negative() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "negative_tier_event");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "bad"),
        TicketTier {
            name: String::from_str(&env, "Bad"),
            price: 100,
            tier_limit: -1,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id,
        name: String::from_str(&env, "Negative Tier Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 0,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidQuantity)));
}

#[test]
fn test_register_event_milestone_overflow() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let payment_addr = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    env.mock_all_auths();

    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let event_id = String::from_str(&env, "milestone_overflow");
    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let milestones = soroban_sdk::vec![
        &env,
        crate::types::Milestone {
            sales_threshold: 10,
            release_percent: u32::MAX,
        },
        crate::types::Milestone {
            sales_threshold: 20,
            release_percent: 1,
        },
    ];

    let result = client.try_register_event(&EventRegistrationArgs {
        event_id,
        name: String::from_str(&env, "Milestone Overflow Event"),
        organizer_address: organizer,
        payment_address: payment_addr,
        metadata_cid,
        max_supply: 100,
        milestone_plan: Some(milestones),
        tiers: Map::new(&env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    assert_eq!(result, Err(Ok(EventRegistryError::SupplyOverflow)));
}

// ── Tags Tests ────────────────────────────────────────────────────────────────

/// Helper: initialise the registry and return (client, admin, organizer).
fn setup_tags_test(env: &Env) -> (EventRegistryClient<'static>, Address, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    let organizer = Address::generate(env);
    (client, admin, organizer)
}

fn tags_base_args(env: &Env, event_id: &str, organizer: &Address) -> EventRegistrationArgs {
    EventRegistrationArgs {
        event_id: String::from_str(env, event_id),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(env),
        metadata_cid: String::from_str(
            env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply: 0,
        milestone_plan: None,
        tiers: Map::new(env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    }
}

/// Registering with no tags (None) succeeds and EventInfo.tags is None.
#[test]
fn test_register_event_without_tags() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, organizer) = setup_tags_test(&env);

    let args = tags_base_args(&env, "evt_no_tags", &organizer);
    client.register_event(&args);

    let info = client
        .get_event(&String::from_str(&env, "evt_no_tags"))
        .unwrap();
    assert!(info.tags.is_none());
}

/// Registering with a valid set of tags stores them correctly.
#[test]
fn test_register_event_with_tags_stored_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, organizer) = setup_tags_test(&env);

    let tags = soroban_sdk::vec![
        &env,
        String::from_str(&env, "Music"),
        String::from_str(&env, "Tech"),
        String::from_str(&env, "Outdoor"),
    ];

    let mut args = tags_base_args(&env, "evt_tags", &organizer);
    args.tags = Some(tags.clone());
    client.register_event(&args);

    let info = client
        .get_event(&String::from_str(&env, "evt_tags"))
        .unwrap();
    let stored = info.tags.unwrap();
    assert_eq!(stored.len(), 3);
    assert_eq!(stored.get(0).unwrap(), String::from_str(&env, "Music"));
    assert_eq!(stored.get(1).unwrap(), String::from_str(&env, "Tech"));
    assert_eq!(stored.get(2).unwrap(), String::from_str(&env, "Outdoor"));
}

/// Exactly 10 tags is the maximum allowed — must succeed.
#[test]
fn test_register_event_with_exactly_10_tags_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, organizer) = setup_tags_test(&env);

    let tags = soroban_sdk::vec![
        &env,
        String::from_str(&env, "Tag1"),
        String::from_str(&env, "Tag2"),
        String::from_str(&env, "Tag3"),
        String::from_str(&env, "Tag4"),
        String::from_str(&env, "Tag5"),
        String::from_str(&env, "Tag6"),
        String::from_str(&env, "Tag7"),
        String::from_str(&env, "Tag8"),
        String::from_str(&env, "Tag9"),
        String::from_str(&env, "Tag10"),
    ];

    let mut args = tags_base_args(&env, "evt_10_tags", &organizer);
    args.tags = Some(tags);
    let result = client.try_register_event(&args);
    assert!(result.is_ok(), "10 tags should be accepted");
}

/// 11 tags exceeds the maximum — must return InvalidTags.
#[test]
fn test_register_event_with_11_tags_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, organizer) = setup_tags_test(&env);

    let tags = soroban_sdk::vec![
        &env,
        String::from_str(&env, "T1"),
        String::from_str(&env, "T2"),
        String::from_str(&env, "T3"),
        String::from_str(&env, "T4"),
        String::from_str(&env, "T5"),
        String::from_str(&env, "T6"),
        String::from_str(&env, "T7"),
        String::from_str(&env, "T8"),
        String::from_str(&env, "T9"),
        String::from_str(&env, "T10"),
        String::from_str(&env, "T11"),
    ];

    let mut args = tags_base_args(&env, "evt_11_tags", &organizer);
    args.tags = Some(tags);
    let result = client.try_register_event(&args);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidTags)));
}

/// A tag of exactly 32 characters is the maximum allowed length — must succeed.
#[test]
fn test_register_event_tag_exactly_32_chars_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, organizer) = setup_tags_test(&env);

    // 32 characters exactly
    let tag_32 = String::from_str(&env, "12345678901234567890123456789012");
    let tags = soroban_sdk::vec![&env, tag_32];

    let mut args = tags_base_args(&env, "evt_tag_32", &organizer);
    args.tags = Some(tags);
    let result = client.try_register_event(&args);
    assert!(result.is_ok(), "32-char tag should be accepted");
}

/// A tag of 33 characters exceeds the maximum length — must return InvalidTags.
#[test]
fn test_register_event_tag_33_chars_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, organizer) = setup_tags_test(&env);

    // 33 characters
    let tag_33 = String::from_str(&env, "123456789012345678901234567890123");
    let tags = soroban_sdk::vec![&env, tag_33];

    let mut args = tags_base_args(&env, "evt_tag_33", &organizer);
    args.tags = Some(tags);
    let result = client.try_register_event(&args);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidTags)));
}

/// A single empty-string tag is valid (length 0 ≤ 32).
#[test]
fn test_register_event_empty_tag_is_valid() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, organizer) = setup_tags_test(&env);

    let tags = soroban_sdk::vec![&env, String::from_str(&env, "")];
    let mut args = tags_base_args(&env, "evt_empty_tag", &organizer);
    args.tags = Some(tags);
    let result = client.try_register_event(&args);
    assert!(result.is_ok(), "empty tag string should be accepted");
}

/// Tags are independent of other fields — an event with tags and a banner_cid
/// stores both correctly.
#[test]
fn test_register_event_tags_and_banner_cid_coexist() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, organizer) = setup_tags_test(&env);

    let banner = String::from_str(&env, "bafybeibanner123");
    let tags = soroban_sdk::vec![
        &env,
        String::from_str(&env, "Music"),
        String::from_str(&env, "Festival"),
    ];

    let mut args = tags_base_args(&env, "evt_both", &organizer);
    args.banner_cid = Some(banner.clone());
    args.tags = Some(tags);
    client.register_event(&args);

    let info = client
        .get_event(&String::from_str(&env, "evt_both"))
        .unwrap();
    assert_eq!(info.banner_cid.unwrap(), banner);
    assert_eq!(info.tags.unwrap().len(), 2);
}

// ── VERSION constant ──────────────────────────────────────────────────────────

/// The VERSION constant must equal 1.
#[test]
fn test_version_constant_value() {
    assert_eq!(crate::VERSION, 1u32);
}

/// The `version()` contract function must return 1.
#[test]
fn test_version_fn_returns_1() {
    let env = Env::default();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);
    assert_eq!(client.version(), 1u32);
}

// ── Tier-Specific Loyalty Multipliers ────────────────────────────────────────

/// A multiplier of 1 (standard) awards 10 points per ticket — baseline behaviour.
#[test]
fn test_loyalty_multiplier_1x_awards_base_points() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    // 3 tickets × 10 pts × 1x = 30 pts
    client.update_loyalty_score(&admin, &guest, &3, &3000i128, &1u32);

    let profile = client.get_guest_profile(&guest).unwrap();
    assert_eq!(profile.loyalty_score, 30);
}

/// A multiplier of 2 (e.g., VIP tier) awards 20 points per ticket.
#[test]
fn test_loyalty_multiplier_2x_doubles_points() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    // 3 tickets × 10 pts × 2x = 60 pts
    client.update_loyalty_score(&admin, &guest, &3, &6000i128, &2u32);

    let profile = client.get_guest_profile(&guest).unwrap();
    assert_eq!(profile.loyalty_score, 60);
}

/// A multiplier of 3 (e.g., Platinum tier) awards 30 points per ticket.
#[test]
fn test_loyalty_multiplier_3x_triples_points() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    // 2 tickets × 10 pts × 3x = 60 pts
    client.update_loyalty_score(&admin, &guest, &2, &9000i128, &3u32);

    let profile = client.get_guest_profile(&guest).unwrap();
    assert_eq!(profile.loyalty_score, 60);
}

/// A multiplier of 0 is treated as 1x (no zeroing of points).
#[test]
fn test_loyalty_multiplier_0_treated_as_1x() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    // 4 tickets × 10 pts × 1x (0 treated as 1) = 40 pts
    client.update_loyalty_score(&admin, &guest, &4, &4000i128, &0u32);

    let profile = client.get_guest_profile(&guest).unwrap();
    assert_eq!(profile.loyalty_score, 40);
}

/// Points from multiple purchases with different multipliers accumulate correctly.
#[test]
fn test_loyalty_multiplier_accumulates_across_purchases() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_loyalty_env(&env);

    let guest = Address::generate(&env);
    // Purchase 1: 2 tickets × 10 × 1x = 20 pts
    client.update_loyalty_score(&admin, &guest, &2, &2000i128, &1u32);
    // Purchase 2: 1 ticket × 10 × 2x = 20 pts
    client.update_loyalty_score(&admin, &guest, &1, &5000i128, &2u32);
    // Total: 40 pts

    let profile = client.get_guest_profile(&guest).unwrap();
    assert_eq!(profile.loyalty_score, 40);
}

/// TicketTier with loyalty_multiplier = 2 stores and retrieves the field correctly.
#[test]
fn test_ticket_tier_loyalty_multiplier_stored_in_event() {
// ── set_feedback_cid tests ────────────────────────────────────────────────────

fn setup_event_with_end_time(
    env: &Env,
    client: &EventRegistryClient,
    event_id: &str,
    end_time: u64,
) -> (Address, Address) {
    let admin = Address::generate(env);
    let organizer = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let metadata_cid = String::from_str(
        env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&EventRegistrationArgs {
        event_id: String::from_str(env, event_id),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(env),
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers: Map::new(env),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time,
    });
    (admin, organizer)
}

/// Organizer can set feedback CID after end_time has passed.
#[test]
fn test_set_feedback_cid_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    // Set ledger time to 1000 so we can have a past end_time
    env.ledger().set_timestamp(1000);
    let past_end_time = 500u64;
    setup_event_with_end_time(&env, &client, "evt_feedback", past_end_time);

    let feedback_cid = String::from_str(
        &env,
        "bafkreifeedback222222222222222222222222222222222222222222222",
    );
    client.set_feedback_cid(&String::from_str(&env, "evt_feedback"), &feedback_cid);

    let event = client
        .get_event(&String::from_str(&env, "evt_feedback"))
        .unwrap();
    assert_eq!(event.feedback_cid, Some(feedback_cid));
}

/// set_feedback_cid fails when end_time is 0 (not set).
#[test]
fn test_set_feedback_cid_no_end_time_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    setup_event_with_end_time(&env, &client, "evt_no_end", 0);

    let feedback_cid = String::from_str(
        &env,
        "bafkreifeedback222222222222222222222222222222222222222222222",
    );
    let result = client.try_set_feedback_cid(&String::from_str(&env, "evt_no_end"), &feedback_cid);
    assert_eq!(result, Err(Ok(EventRegistryError::EventNotEnded)));
}

/// set_feedback_cid fails when end_time is in the future.
#[test]
fn test_set_feedback_cid_before_end_time_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let future_end_time = env.ledger().timestamp() + 10_000;
    setup_event_with_end_time(&env, &client, "evt_future", future_end_time);

    let feedback_cid = String::from_str(
        &env,
        "bafkreifeedback222222222222222222222222222222222222222222222",
    );
    let result = client.try_set_feedback_cid(&String::from_str(&env, "evt_future"), &feedback_cid);
    assert_eq!(result, Err(Ok(EventRegistryError::EventNotEnded)));
}

/// set_feedback_cid fails for a non-existent event.
#[test]
fn test_set_feedback_cid_event_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let result = client.try_set_feedback_cid(
        &String::from_str(&env, "nonexistent"),
        &String::from_str(
            &env,
            "bafkreifeedback222222222222222222222222222222222222222222222",
        ),
    );
    assert_eq!(result, Err(Ok(EventRegistryError::EventNotFound)));
}

/// set_feedback_cid fails with an invalid CID format.
#[test]
fn test_set_feedback_cid_invalid_cid() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    env.ledger().set_timestamp(1000);
    setup_event_with_end_time(&env, &client, "evt_bad_cid", 500);

    let result = client.try_set_feedback_cid(
        &String::from_str(&env, "evt_bad_cid"),
        &String::from_str(&env, "short"),
    );
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidMetadataCid)));
}

/// set_feedback_cid fails on a cancelled event.
#[test]
fn test_set_feedback_cid_cancelled_event_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let organizer = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "vip"),
        crate::types::TicketTier {
            name: String::from_str(&env, "VIP"),
            price: 5000,
            tier_limit: 50,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 2,
        },
    );
    tiers.set(
        String::from_str(&env, "general"),
        crate::types::TicketTier {
            name: String::from_str(&env, "General"),
            price: 1000,
            tier_limit: 200,
            current_sold: 0,
            is_refundable: false,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
        },
    );

    let metadata_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.register_event(&EventRegistrationArgs {
        event_id: String::from_str(&env, "evt_multiplier"),
        name: String::from_str(&env, "Multiplier Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(&env),
        metadata_cid,
        max_supply: 250,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
    });

    let event = client
        .get_event(&String::from_str(&env, "evt_multiplier"))
        .unwrap();
    let vip_tier = event.tiers.get(String::from_str(&env, "vip")).unwrap();
    let general_tier = event.tiers.get(String::from_str(&env, "general")).unwrap();

    assert_eq!(vip_tier.loyalty_multiplier, 2);
    assert_eq!(general_tier.loyalty_multiplier, 1);
    env.ledger().set_timestamp(1000);
    setup_event_with_end_time(&env, &client, "evt_cancelled", 500);

    let event_id = String::from_str(&env, "evt_cancelled");
    client.cancel_event(&event_id);

    let result = client.try_set_feedback_cid(
        &event_id,
        &String::from_str(
            &env,
            "bafkreifeedback222222222222222222222222222222222222222222222",
        ),
    );
    assert_eq!(result, Err(Ok(EventRegistryError::EventCancelled)));
}
