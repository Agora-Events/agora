use super::*;
use crate::error::EventRegistryError;
use crate::types::{EventRegistrationArgs, EventStatus, TicketTier};
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String};

/// Helper: initialize the contract and return (client, admin, platform_wallet).
fn setup(env: &Env) -> (EventRegistryClient<'static>, Address, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);

    client.initialize(&admin, &platform_wallet, &500);
    (client, admin, platform_wallet)
}

/// Helper: build an `EventRegistrationArgs` with sensible defaults.
fn make_event_args(
    env: &Env,
    event_id: &str,
    organizer: &Address,
    max_supply: i128,
    tiers: Map<String, TicketTier>,
) -> EventRegistrationArgs {
    EventRegistrationArgs {
        event_id: String::from_str(env, event_id),
        organizer_address: organizer.clone(),
        payment_address: organizer.clone(),
        metadata_cid: String::from_str(
            env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
        max_supply,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
    }
}

/// Helper: build a single-tier map (General, price 1000, given limit).
fn single_tier(env: &Env, tier_limit: i128) -> Map<String, TicketTier> {
    let mut tiers = Map::new(env);
    tiers.set(
        String::from_str(env, "tier_1"),
        TicketTier {
            name: String::from_str(env, "General"),
            price: 1000,
            tier_limit,
            current_sold: 0,
            is_refundable: true,
        },
    );
    tiers
}

// ---------------------------------------------------------------------------
// 1. Complete Event Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_complete_event_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    // Register event
    let args = make_event_args(&env, "evt_1", &organizer, 100, single_tier(&env, 100));
    client.register_event(&args);

    // Verify active
    let info = client.get_event(&String::from_str(&env, "evt_1")).unwrap();
    assert!(info.is_active);
    assert_eq!(info.status, EventStatus::Active);

    // Deactivate
    client.update_event_status(&String::from_str(&env, "evt_1"), &false);
    let info = client.get_event(&String::from_str(&env, "evt_1")).unwrap();
    assert!(!info.is_active);

    // Reactivate
    client.update_event_status(&String::from_str(&env, "evt_1"), &true);
    let info = client.get_event(&String::from_str(&env, "evt_1")).unwrap();
    assert!(info.is_active);

    // Cancel (irreversible)
    client.cancel_event(&String::from_str(&env, "evt_1"));
    let info = client.get_event(&String::from_str(&env, "evt_1")).unwrap();
    assert!(!info.is_active);
    assert_eq!(info.status, EventStatus::Cancelled);

    // Update after cancel should fail
    let result = client.try_update_event_status(&String::from_str(&env, "evt_1"), &true);
    assert_eq!(result, Err(Ok(EventRegistryError::EventCancelled)));

    // Cancel again should fail
    let result = client.try_cancel_event(&String::from_str(&env, "evt_1"));
    assert_eq!(result, Err(Ok(EventRegistryError::EventAlreadyCancelled)));
}

// ---------------------------------------------------------------------------
// 2. Zero max_supply means unlimited
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_zero_max_supply_means_unlimited() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    // Register event with max_supply = 0 (unlimited)
    // Tier limit must also accommodate unlimited; use a large number.
    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "tier_1"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 1000,
            tier_limit: i128::MAX,
            current_sold: 0,
            is_refundable: true,
        },
    );
    let args = make_event_args(&env, "evt_unlim", &organizer, 0, tiers);
    client.register_event(&args);

    // Set ticket payment contract so increment_inventory can be called
    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);

    // Increment multiple times — all should succeed
    for _ in 0..10 {
        client.increment_inventory(
            &String::from_str(&env, "evt_unlim"),
            &String::from_str(&env, "tier_1"),
            &1,
        );
    }

    let info = client
        .get_event(&String::from_str(&env, "evt_unlim"))
        .unwrap();
    assert_eq!(info.current_supply, 10);
}

// ---------------------------------------------------------------------------
// 3. Inventory limits enforced
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_inventory_limits_enforced() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let args = make_event_args(&env, "evt_cap", &organizer, 5, single_tier(&env, 5));
    client.register_event(&args);

    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);

    // Increment 5 times — should succeed
    for _ in 0..5 {
        client.increment_inventory(
            &String::from_str(&env, "evt_cap"),
            &String::from_str(&env, "tier_1"),
            &1,
        );
    }

    let info = client
        .get_event(&String::from_str(&env, "evt_cap"))
        .unwrap();
    assert_eq!(info.current_supply, 5);

    // 6th increment should fail with MaxSupplyExceeded
    let result = client.try_increment_inventory(
        &String::from_str(&env, "evt_cap"),
        &String::from_str(&env, "tier_1"),
        &1,
    );
    assert_eq!(result, Err(Ok(EventRegistryError::MaxSupplyExceeded)));
}

// ---------------------------------------------------------------------------
// 4. Tier supply limits
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_tier_supply_limits() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    // Event with max_supply=10 but tier_limit=3
    let args = make_event_args(&env, "evt_tier", &organizer, 10, single_tier(&env, 3));
    client.register_event(&args);

    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);

    // 3 increments succeed
    for _ in 0..3 {
        client.increment_inventory(
            &String::from_str(&env, "evt_tier"),
            &String::from_str(&env, "tier_1"),
            &1,
        );
    }

    // 4th fails with TierSupplyExceeded
    let result = client.try_increment_inventory(
        &String::from_str(&env, "evt_tier"),
        &String::from_str(&env, "tier_1"),
        &1,
    );
    assert_eq!(result, Err(Ok(EventRegistryError::TierSupplyExceeded)));
}

// ---------------------------------------------------------------------------
// 5. Blacklist suspends and blocks
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_blacklist_suspends_and_blocks() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    // Register an event
    let args = make_event_args(&env, "evt_bl", &organizer, 100, single_tier(&env, 100));
    client.register_event(&args);
    let info = client.get_event(&String::from_str(&env, "evt_bl")).unwrap();
    assert!(info.is_active);

    // Blacklist organizer
    client.blacklist_organizer(&organizer, &String::from_str(&env, "Fraud detected"));
    assert!(client.is_organizer_blacklisted(&organizer));

    // Event should be suspended (is_active = false)
    let info = client.get_event(&String::from_str(&env, "evt_bl")).unwrap();
    assert!(!info.is_active);

    // Try to register a new event — should fail
    let args2 = make_event_args(&env, "evt_bl2", &organizer, 50, single_tier(&env, 50));
    let result = client.try_register_event(&args2);
    assert_eq!(result, Err(Ok(EventRegistryError::OrganizerBlacklisted)));

    // Remove from blacklist
    client.remove_from_blacklist(&organizer, &String::from_str(&env, "Cleared after review"));
    assert!(!client.is_organizer_blacklisted(&organizer));

    // Now registering a new event should succeed
    let args3 = make_event_args(&env, "evt_bl3", &organizer, 50, single_tier(&env, 50));
    client.register_event(&args3);
    assert!(client.event_exists(&String::from_str(&env, "evt_bl3")));
}

// ---------------------------------------------------------------------------
// 6. Inventory decrement after increment
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_inventory_decrement_after_increment() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let args = make_event_args(&env, "evt_dec", &organizer, 10, single_tier(&env, 10));
    client.register_event(&args);

    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "evt_dec");
    let tier_id = String::from_str(&env, "tier_1");

    // Increment 3
    client.increment_inventory(&event_id, &tier_id, &3);
    let info = client.get_event(&event_id).unwrap();
    assert_eq!(info.current_supply, 3);

    // Decrement 1
    client.decrement_inventory(&event_id, &tier_id);
    let info = client.get_event(&event_id).unwrap();
    assert_eq!(info.current_supply, 2);

    // Decrement to 0
    client.decrement_inventory(&event_id, &tier_id);
    client.decrement_inventory(&event_id, &tier_id);
    let info = client.get_event(&event_id).unwrap();
    assert_eq!(info.current_supply, 0);

    // Further decrement should fail (underflow)
    let result = client.try_decrement_inventory(&event_id, &tier_id);
    assert_eq!(result, Err(Ok(EventRegistryError::SupplyUnderflow)));
}

// ---------------------------------------------------------------------------
// 7. Minimum Goal Tracking
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_min_goal_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let mut args = make_event_args(&env, "evt_goal", &organizer, 100, single_tier(&env, 100));
    args.min_sales_target = Some(10);
    args.target_deadline = Some(1000);
    client.register_event(&args);

    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);

    let event_id = String::from_str(&env, "evt_goal");
    let tier_id = String::from_str(&env, "tier_1");

    let info = client.get_event(&event_id).unwrap();
    assert!(!info.goal_met);
    assert_eq!(info.min_sales_target, 10);

    // Increment 5 - goal still not met
    client.increment_inventory(&event_id, &tier_id, &5);
    let info = client.get_event(&event_id).unwrap();
    assert!(!info.goal_met);

    // Increment 5 more - goal should be met
    client.increment_inventory(&event_id, &tier_id, &5);
    let info = client.get_event(&event_id).unwrap();
    assert!(info.goal_met);

    // Further increments keep it met
    client.increment_inventory(&event_id, &tier_id, &1);
    let info = client.get_event(&event_id).unwrap();
    assert!(info.goal_met);
}
