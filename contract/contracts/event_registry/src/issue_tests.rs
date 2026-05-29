use crate::error::EventRegistryError;
use crate::types::{EventInfo, EventRegistrationArgs, EventStatus, TicketTier};
use crate::{storage, EventRegistry, EventRegistryClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

fn setup(env: &Env) -> (EventRegistryClient<'static>, Address, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    (client, admin, contract_id)
}

fn metadata_cid(env: &Env) -> String {
    String::from_str(
        env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    )
}

fn event_args(env: &Env, event_id: &str, organizer: &Address) -> EventRegistrationArgs {
    let mut tiers = Map::new(env);
    tiers.set(
        String::from_str(env, "general"),
        TicketTier {
            name: String::from_str(env, "General"),
            price: 1000,
            tier_limit: 100,
            current_sold: 0,
            is_refundable: true,
            auction_config: Vec::new(env),
            loyalty_multiplier: 1,
            max_per_user: 0,
        },
    );

    EventRegistrationArgs {
        event_id: String::from_str(env, event_id),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: Address::generate(env),
        metadata_cid: metadata_cid(env),
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
        category_ids: None,
        start_time: 0,
        is_private: false,
        end_time: 0,
        transfer_lock_duration: 0,
        accepted_tokens: Vec::new(env),
        use_global_whitelist: true,
        referral_rate_bps: None,
    }
}

fn store_event_without_auth(env: &Env, contract_id: &Address, event_id: &str, organizer: &Address) {
    let args = event_args(env, event_id, organizer);
    let event = EventInfo {
        event_id: args.event_id,
        name: args.name,
        organizer_address: args.organizer_address,
        payment_address: args.payment_address,
        platform_fee_percent: 500,
        is_active: true,
        status: EventStatus::Active,
        created_at: env.ledger().timestamp(),
        metadata_cid: args.metadata_cid,
        max_supply: args.max_supply,
        current_supply: 0,
        milestone_plan: args.milestone_plan,
        tiers: args.tiers,
        refund_deadline: args.refund_deadline,
        restocking_fee: args.restocking_fee,
        resale_cap_bps: args.resale_cap_bps,
        is_postponed: false,
        grace_period_end: 0,
        min_sales_target: 0,
        target_deadline: 0,
        goal_met: false,
        custom_fee_bps: None,
        banner_cid: args.banner_cid,
        tags: args.tags,
        category_ids: args.category_ids,
        start_time: args.start_time,
        is_private: args.is_private,
        end_time: args.end_time,
        transfer_lock_duration: args.transfer_lock_duration,
        accepted_tokens: args.accepted_tokens,
        use_global_whitelist: args.use_global_whitelist,
        feedback_cid: None,
        cancellation_reason: None,
        referral_rate_bps: args.referral_rate_bps.unwrap_or(0),
    };

    env.as_contract(contract_id, || storage::store_event(env, event));
}

#[test]
fn test_blacklist_organizer_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);

    client.blacklist_organizer(&organizer, &String::from_str(&env, "fraud"));

    let result = client.try_register_event(&event_args(&env, "blacklisted_event", &organizer));
    assert_eq!(result, Err(Ok(EventRegistryError::OrganizerBlacklisted)));
}

#[test]
fn test_remove_from_blacklist() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);

    client.blacklist_organizer(&organizer, &String::from_str(&env, "review"));
    client.remove_from_blacklist(&organizer, &String::from_str(&env, "cleared"));
    client.register_event(&event_args(&env, "restored_event", &organizer));

    assert!(client
        .get_event(&String::from_str(&env, "restored_event"))
        .is_some());
}

#[test]
fn test_blacklist_already_blacklisted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);

    client.blacklist_organizer(&organizer, &String::from_str(&env, "first"));
    client.blacklist_organizer(&organizer, &String::from_str(&env, "second"));

    assert!(client.is_organizer_blacklisted(&organizer));
    assert_eq!(client.get_blacklist_audit_log().len(), 1);
}

#[test]
fn test_remove_not_blacklisted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);

    let result = client.try_remove_from_blacklist(&organizer, &String::from_str(&env, "none"));
    assert_eq!(result, Err(Ok(EventRegistryError::OrganizerNotBlacklisted)));
}

#[test]
fn test_authorize_scanner_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "scanner_event");

    client.register_event(&event_args(&env, "scanner_event", &organizer));
    client.authorize_scanner(&event_id, &scanner);

    assert!(client.is_scanner_authorized(&event_id, &scanner));
}

#[test]
fn test_revoke_scanner() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "revoke_scanner_event");

    client.register_event(&event_args(&env, "revoke_scanner_event", &organizer));
    client.authorize_scanner(&event_id, &scanner);
    client.revoke_scanner(&event_id, &scanner);

    assert!(!client.is_scanner_authorized(&event_id, &scanner));
}

#[test]
#[should_panic]
fn test_authorize_scanner_unauthorized() {
    let env = Env::default();
    let (client, _admin, contract_id) = setup(&env);
    let organizer = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "unauthorized_scanner_event");

    store_event_without_auth(&env, &contract_id, "unauthorized_scanner_event", &organizer);
    client.authorize_scanner(&event_id, &scanner);
}

#[test]
fn test_scanner_for_nonexistent_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let scanner = Address::generate(&env);

    let result = client.try_authorize_scanner(&String::from_str(&env, "missing"), &scanner);
    assert_eq!(result, Err(Ok(EventRegistryError::EventNotFound)));
}

#[test]
fn test_set_global_promo_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let expiry = env.ledger().timestamp() + 100;

    client.set_global_promo(&1500, &expiry);

    assert_eq!(client.get_global_promo_bps(), 1500);
    assert_eq!(client.get_promo_expiry(), expiry);
}

#[test]
fn test_set_global_promo_invalid_bps() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);

    let result = client.try_set_global_promo(&10001, &(env.ledger().timestamp() + 100));
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidPromoBps)));
}

#[test]
fn test_set_global_promo_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let expired_at = env.ledger().timestamp().saturating_sub(1);

    client.set_global_promo(&2500, &expired_at);

    assert_eq!(client.get_global_promo_bps(), 0);
}

#[test]
#[should_panic]
fn test_set_global_promo_unauthorized() {
    let env = Env::default();
    let (client, _admin, _contract_id) = setup(&env);

    client.set_global_promo(&1000, &(env.ledger().timestamp() + 100));
}
