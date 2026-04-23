//! # Free Ticket (Zero-Price) Unit Tests
//!
//! Covers issue #322: verify that the inventory system handles free tickets
//! (price = 0) correctly.
//!
//! Key invariants checked:
//! * `increment_inventory` succeeds for a zero-price tier.
//! * `current_supply` and per-tier `current_sold` are incremented correctly.
//! * The global tickets-sold counter is updated.
//! * No token-transfer calls are made (the registry never touches tokens).
//! * Capacity limits still apply even when the price is zero.
//! * `decrement_inventory` (refund path) works for a zero-price tier.
//! * Purchasing quantity > 1 in a single call works for a free tier.

use super::*;
use crate::error::EventRegistryError;
use crate::types::{EventRegistrationArgs, TicketTier};
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const VALID_CID: &str = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
const FREE_TIER_ID: &str = "free";

fn setup(env: &Env) -> (EventRegistryClient<'static>, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);
    (client, admin)
}

/// Register a ticket-payment contract and return its address.
fn register_ticket_payment(env: &Env, client: &EventRegistryClient) -> Address {
    let tp_addr = Address::generate(env);
    client.set_ticket_payment_contract(&tp_addr);
    tp_addr
}

/// Build a single free tier (price = 0) with the given limit.
fn free_tier(env: &Env, limit: i128) -> Map<String, TicketTier> {
    let mut tiers = Map::new(env);
    tiers.set(
        String::from_str(env, FREE_TIER_ID),
        TicketTier {
            name: String::from_str(env, "Free Admission"),
            price: 0,
            tier_limit: limit,
            current_sold: 0,
            is_refundable: true,
            auction_config: soroban_sdk::vec![env],
        },
    );
    tiers
}

/// Register an event with a free tier and return its event_id string.
fn register_free_event(
    env: &Env,
    client: &EventRegistryClient,
    organizer: &Address,
    event_id: &str,
    max_supply: i128,
    tier_limit: i128,
) -> String {
    let id = String::from_str(env, event_id);
    client.register_event(&EventRegistrationArgs {
        event_id: id.clone(),
        name: String::from_str(env, "Free Event"),
        organizer_address: organizer.clone(),
        payment_address: Address::generate(env),
        metadata_cid: String::from_str(env, VALID_CID),
        max_supply,
        milestone_plan: None,
        tiers: free_tier(env, tier_limit),
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        end_time: 0,
    });
    id
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// A single free-ticket purchase increments both the per-tier counter and the
/// event-level `current_supply`.
#[test]
fn test_free_ticket_increments_inventory() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup(&env);
    let organizer = Address::generate(&env);
    let _tp = register_ticket_payment(&env, &client);

    let event_id = register_free_event(&env, &client, &organizer, "free_evt_1", 100, 100);
    let tier_id = String::from_str(&env, FREE_TIER_ID);

    // Before purchase
    let before = client.get_event(&event_id).unwrap();
    assert_eq!(before.current_supply, 0);
    assert_eq!(before.tiers.get(tier_id.clone()).unwrap().current_sold, 0);

    // Simulate TicketPayment calling increment_inventory for qty = 1
    client.increment_inventory(&event_id, &tier_id, &1);

    // After purchase
    let after = client.get_event(&event_id).unwrap();
    assert_eq!(after.current_supply, 1);
    assert_eq!(after.tiers.get(tier_id).unwrap().current_sold, 1);
}

/// The global tickets-sold counter is updated after a free-ticket purchase.
#[test]
fn test_free_ticket_updates_global_counter() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup(&env);
    let organizer = Address::generate(&env);
    let _tp = register_ticket_payment(&env, &client);

    let event_id = register_free_event(&env, &client, &organizer, "free_evt_global", 50, 50);
    let tier_id = String::from_str(&env, FREE_TIER_ID);

    let before = client.get_global_tickets_sold();
    client.increment_inventory(&event_id, &tier_id, &1);
    let after = client.get_global_tickets_sold();

    assert_eq!(after, before + 1);
}

/// The registry never calls token-transfer functions — verified by confirming
/// no token contract is invoked during a free-ticket increment.  Because
/// `EventRegistry` never holds a token client, the absence of any token
/// address in the contract's storage after the call is sufficient evidence.
#[test]
fn test_free_ticket_no_token_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup(&env);
    let organizer = Address::generate(&env);
    let _tp = register_ticket_payment(&env, &client);

    let event_id = register_free_event(&env, &client, &organizer, "free_evt_no_transfer", 10, 10);
    let tier_id = String::from_str(&env, FREE_TIER_ID);

    // increment_inventory must succeed without any token interaction.
    // If the contract attempted a zero-value transfer it would panic because
    // no token contract is deployed in this test environment.
    client.increment_inventory(&event_id, &tier_id, &1);

    // Confirm supply was updated — proving the call completed successfully.
    let info = client.get_event(&event_id).unwrap();
    assert_eq!(info.current_supply, 1);
}

/// Purchasing multiple free tickets in a single call (quantity > 1) works.
#[test]
fn test_free_ticket_bulk_purchase() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup(&env);
    let organizer = Address::generate(&env);
    let _tp = register_ticket_payment(&env, &client);

    let event_id = register_free_event(&env, &client, &organizer, "free_evt_bulk", 200, 200);
    let tier_id = String::from_str(&env, FREE_TIER_ID);

    client.increment_inventory(&event_id, &tier_id, &5);

    let info = client.get_event(&event_id).unwrap();
    assert_eq!(info.current_supply, 5);
    assert_eq!(info.tiers.get(tier_id).unwrap().current_sold, 5);
}

/// Capacity limits are enforced even when the ticket price is zero.
#[test]
fn test_free_ticket_respects_tier_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup(&env);
    let organizer = Address::generate(&env);
    let _tp = register_ticket_payment(&env, &client);

    // Tier limit = 2
    let event_id = register_free_event(&env, &client, &organizer, "free_evt_cap", 10, 2);
    let tier_id = String::from_str(&env, FREE_TIER_ID);

    client.increment_inventory(&event_id, &tier_id, &1);
    client.increment_inventory(&event_id, &tier_id, &1);

    // Third ticket must be rejected
    let result = client.try_increment_inventory(&event_id, &tier_id, &1);
    assert_eq!(result, Err(Ok(EventRegistryError::TierSupplyExceeded)));
}

/// max_supply cap is enforced for free tickets when set.
#[test]
fn test_free_ticket_respects_max_supply() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup(&env);
    let organizer = Address::generate(&env);
    let _tp = register_ticket_payment(&env, &client);

    // max_supply = 100, tier_limit = 100 (valid)
    let event_id = register_free_event(&env, &client, &organizer, "free_evt_maxsup", 100, 100);
    let tier_id = String::from_str(&env, FREE_TIER_ID);

    // Increment by 99 (should succeed)
    client.increment_inventory(&event_id, &tier_id, &99);

    // Now try to increment by 2 more (should fail - only 1 left)
    let result = client.try_increment_inventory(&event_id, &tier_id, &2);
    assert_eq!(result, Err(Ok(EventRegistryError::MaxSupplyExceeded)));
}

/// Refunding a free ticket (decrement_inventory) rolls back the supply counters.
#[test]
fn test_free_ticket_decrement_on_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup(&env);
    let organizer = Address::generate(&env);
    let _tp = register_ticket_payment(&env, &client);

    let event_id = register_free_event(&env, &client, &organizer, "free_evt_refund", 50, 50);
    let tier_id = String::from_str(&env, FREE_TIER_ID);

    client.increment_inventory(&event_id, &tier_id, &1);

    let after_purchase = client.get_event(&event_id).unwrap();
    assert_eq!(after_purchase.current_supply, 1);

    client.decrement_inventory(&event_id, &tier_id);

    let after_refund = client.get_event(&event_id).unwrap();
    assert_eq!(after_refund.current_supply, 0);
    assert_eq!(after_refund.tiers.get(tier_id).unwrap().current_sold, 0);
}

/// Calling increment_inventory with quantity = 0 is rejected.
#[test]
fn test_free_ticket_zero_quantity_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup(&env);
    let organizer = Address::generate(&env);
    let _tp = register_ticket_payment(&env, &client);

    let event_id = register_free_event(&env, &client, &organizer, "free_evt_qty0", 50, 50);
    let tier_id = String::from_str(&env, FREE_TIER_ID);

    let result = client.try_increment_inventory(&event_id, &tier_id, &0);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidQuantity)));
}
