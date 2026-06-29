use super::*;
use crate::error::EventRegistryError;
use crate::types::{EventRegistrationArgs, EventStatus, TicketTier};
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String};

fn test_payment_address(env: &Env) -> Address {
    Address::from_string(&String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJXFF",
    ))
}

fn setup(env: &Env) -> (EventRegistryClient<'static>, Address, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    (client, admin, platform_wallet)
}

fn make_event_args(
    env: &Env,
    event_id: &str,
    organizer: &Address,
    max_supply: i128,
    tiers: Map<String, TicketTier>,
) -> EventRegistrationArgs {
    EventRegistrationArgs {
        event_id: String::from_str(env, event_id),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: test_payment_address(env),
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
    }
}

fn single_tier(env: &Env, tier_limit: i128, max_per_user: u32) -> Map<String, TicketTier> {
    let mut tiers = Map::new(env);
    tiers.set(
        String::from_str(env, "tier_1"),
        TicketTier {
            name: String::from_str(env, "General"),
            price: 1000,
            tier_limit,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
            max_per_user,
        },
    );
    tiers
}

// ---------------------------------------------------------------------------
// #844: Per-user limit enforcement
// ---------------------------------------------------------------------------

#[test]
fn test_per_user_limit_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let args = make_event_args(&env, "evt_peruser", &organizer, 100, single_tier(&env, 100, 2));
    client.register_event(&args);

    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);

    let user = Address::generate(&env);

    // First purchase of 2 should succeed
    client.increment_inventory(
        &String::from_str(&env, "evt_peruser"),
        &String::from_str(&env, "tier_1"),
        &user,
        &2,
    );

    // Second purchase of 1 should fail (2 + 1 > 2)
    let result = client.try_increment_inventory(
        &String::from_str(&env, "evt_peruser"),
        &String::from_str(&env, "tier_1"),
        &user,
        &1,
    );
    assert_eq!(result, Err(Ok(EventRegistryError::PerUserLimitExceeded)));
}

#[test]
fn test_per_user_limit_zero_means_unlimited() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let args = make_event_args(&env, "evt_unlim_user", &organizer, 100, single_tier(&env, 100, 0));
    client.register_event(&args);

    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);

    let user = Address::generate(&env);

    // Multiple purchases should all succeed when max_per_user == 0
    for _ in 0..10 {
        client.increment_inventory(
            &String::from_str(&env, "evt_unlim_user"),
            &String::from_str(&env, "tier_1"),
            &user,
            &1,
        );
    }
}

// ---------------------------------------------------------------------------
// #845: GlobalTicketsSold counter
// ---------------------------------------------------------------------------

#[test]
fn test_global_tickets_sold_increments() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let args = make_event_args(&env, "evt_global1", &organizer, 100, single_tier(&env, 100, 0));
    client.register_event(&args);

    let ticket_payment = Address::generate(&env);
    client.set_ticket_payment_contract(&ticket_payment);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Initially 0
    assert_eq!(client.get_global_tickets_sold(), 0);

    // Purchase 3 tickets
    client.increment_inventory(
        &String::from_str(&env, "evt_global1"),
        &String::from_str(&env, "tier_1"),
        &user1,
        &3,
    );
    assert_eq!(client.get_global_tickets_sold(), 3);

    // Purchase 2 more tickets
    client.increment_inventory(
        &String::from_str(&env, "evt_global1"),
        &String::from_str(&env, "tier_1"),
        &user2,
        &2,
    );
    assert_eq!(client.get_global_tickets_sold(), 5);

    // Register a second event and purchase tickets there too
    let args2 = make_event_args(&env, "evt_global2", &organizer, 100, single_tier(&env, 100, 0));
    client.register_event(&args2);

    client.increment_inventory(
        &String::from_str(&env, "evt_global2"),
        &String::from_str(&env, "tier_1"),
        &user1,
        &4,
    );
    assert_eq!(client.get_global_tickets_sold(), 9);
}

// ---------------------------------------------------------------------------
// #843: register_event time validation
// ---------------------------------------------------------------------------

#[test]
fn test_register_event_rejects_invalid_time_range() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "tier_1"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 1000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
            max_per_user: 0,
        },
    );

    let mut args = make_event_args(&env, "evt_bad_time", &organizer, 100, tiers);
    args.start_time = 1000;
    args.end_time = 500; // end before start

    let result = client.try_register_event(&args);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidDeadline)));
}

#[test]
fn test_register_event_accepts_valid_time_range() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let mut tiers = Map::new(&env);
    tiers.set(
        String::from_str(&env, "tier_1"),
        TicketTier {
            name: String::from_str(&env, "General"),
            price: 1000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![&env],
            loyalty_multiplier: 1,
            max_per_user: 0,
        },
    );

    let mut args = make_event_args(&env, "evt_good_time", &organizer, 100, tiers);
    args.start_time = 1000;
    args.end_time = 2000; // end after start

    client.register_event(&args);
    let info = client.get_event(&String::from_str(&env, "evt_good_time")).unwrap();
    assert_eq!(info.start_time, 1000);
    assert_eq!(info.end_time, 2000);
}

// ---------------------------------------------------------------------------
// #842: cancel_event accepts reason parameter
// ---------------------------------------------------------------------------

#[test]
fn test_cancel_event_stores_reason() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let args = make_event_args(&env, "evt_cancel", &organizer, 100, single_tier(&env, 100, 0));
    client.register_event(&args);

    let reason = String::from_str(&env, "Venue became unavailable");
    client.cancel_event(&String::from_str(&env, "evt_cancel"), &Some(reason.clone()));

    let info = client
        .get_event(&String::from_str(&env, "evt_cancel"))
        .unwrap();
    assert_eq!(info.status, EventStatus::Cancelled);
    assert_eq!(info.cancellation_reason, Some(reason));
    assert!(!info.is_active);
}

#[test]
fn test_cancel_event_without_reason() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _) = setup(&env);
    let organizer = Address::generate(&env);

    let args = make_event_args(&env, "evt_cancel_none", &organizer, 100, single_tier(&env, 100, 0));
    client.register_event(&args);

    client.cancel_event(&String::from_str(&env, "evt_cancel_none"), &None);

    let info = client
        .get_event(&String::from_str(&env, "evt_cancel_none"))
        .unwrap();
    assert_eq!(info.status, EventStatus::Cancelled);
    assert_eq!(info.cancellation_reason, None);
    assert!(!info.is_active);
}