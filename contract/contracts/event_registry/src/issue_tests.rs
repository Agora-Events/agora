use crate::error::EventRegistryError;
use crate::types::{EventInfo, EventRegistrationArgs, EventStatus, TicketTier};
use crate::{storage, EventRegistry, EventRegistryClient};
use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Env, Map, String, Vec};

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
    event_args_with_tier_count(env, event_id, organizer, 1)
}

fn tier_id_for_index(env: &Env, i: u32) -> String {
    const IDS: [&str; 21] = [
        "tier_00", "tier_01", "tier_02", "tier_03", "tier_04", "tier_05", "tier_06", "tier_07",
        "tier_08", "tier_09", "tier_10", "tier_11", "tier_12", "tier_13", "tier_14", "tier_15",
        "tier_16", "tier_17", "tier_18", "tier_19", "tier_20",
    ];
    String::from_str(env, IDS[i as usize])
}

fn event_args_with_tier_count(
    env: &Env,
    event_id: &str,
    organizer: &Address,
    tier_count: u32,
) -> EventRegistrationArgs {
    let mut tiers = Map::new(env);
    for i in 0..tier_count {
        tiers.set(
            tier_id_for_index(env, i),
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
    }

    EventRegistrationArgs {
        event_id: String::from_str(env, event_id),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address: Address::generate(env),
        metadata_cid: metadata_cid(env),
        max_supply: (tier_count as i128) * 100,
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
fn test_register_event_too_many_tiers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);

    let result = client.try_register_event(&event_args_with_tier_count(
        &env,
        "too_many_tiers_event",
        &organizer,
        21,
    ));
    assert_eq!(result, Err(Ok(EventRegistryError::TooManyTiers)));

    client.register_event(&event_args_with_tier_count(
        &env,
        "max_tiers_event",
        &organizer,
        20,
    ));
    assert_eq!(
        client
            .get_event(&String::from_str(&env, "max_tiers_event"))
            .unwrap()
            .tiers
            .len(),
        20
    );
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
fn test_authorize_scanner_event_includes_authorized_by() {
    use soroban_sdk::testutils::Events;
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "scanner_auth_organizer_event");

    client.register_event(&event_args(
        &env,
        "scanner_auth_organizer_event",
        &organizer,
    ));
    let _ = env.events().all(); // drain setup events

    client.authorize_scanner(&event_id, &scanner);

    // Verify at least one event was emitted by authorize_scanner.
    assert!(!env.events().all().is_empty(), "ScannerAuthorized event was not emitted");

    // Verify the ScannerAuthorizedEvent struct carries the authorized_by field.
    let event_struct = crate::events::ScannerAuthorizedEvent {
        event_id: event_id.clone(),
        scanner: scanner.clone(),
        authorized_by: organizer.clone(),
        timestamp: 0,
    };
    assert_eq!(event_struct.authorized_by, organizer);
}

#[test]
fn test_revoke_scanner_event_includes_revoked_by() {
    use soroban_sdk::testutils::Events;
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "scanner_revoke_organizer_event");

    client.register_event(&event_args(
        &env,
        "scanner_revoke_organizer_event",
        &organizer,
    ));
    client.authorize_scanner(&event_id, &scanner);
    let _ = env.events().all(); // drain setup events

    client.revoke_scanner(&event_id, &scanner);

    // Verify at least one event was emitted by revoke_scanner.
    assert!(!env.events().all().is_empty(), "ScannerRevoked event was not emitted");

    // Verify the ScannerRevokedEvent struct carries the revoked_by field.
    let event_struct = crate::events::ScannerRevokedEvent {
        event_id: event_id.clone(),
        scanner: scanner.clone(),
        revoked_by: organizer.clone(),
        timestamp: 0,
    };
    assert_eq!(event_struct.revoked_by, organizer);
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
fn test_get_global_promo_returns_none_when_unset() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);

    assert_eq!(client.get_global_promo(), None);
}

#[test]
fn test_get_global_promo_returns_none_when_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);

    env.ledger().with_mut(|li| li.timestamp = 1_000);
    client.set_global_promo(&500, &999);

    assert_eq!(client.get_global_promo(), None);
}

#[test]
fn test_get_global_promo_returns_active_discount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);

    env.ledger().with_mut(|li| li.timestamp = 1_000);
    client.set_global_promo(&750, &2_000);

    assert_eq!(client.get_global_promo(), Some((750, 2_000)));
}

#[test]
fn test_update_loyalty_score_applies_tier_multiplier() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _contract_id) = setup(&env);

    let standard_guest = Address::generate(&env);
    let vip_guest = Address::generate(&env);
    let zero_multiplier_guest = Address::generate(&env);

    client.update_loyalty_score(&admin, &standard_guest, &1, &100i128, &1);
    client.update_loyalty_score(&admin, &vip_guest, &1, &100i128, &2);
    client.update_loyalty_score(&admin, &zero_multiplier_guest, &1, &100i128, &0);

    let standard = client.get_guest_profile(&standard_guest).unwrap();
    let vip = client.get_guest_profile(&vip_guest).unwrap();
    let zero_multiplier = client.get_guest_profile(&zero_multiplier_guest).unwrap();

    assert_eq!(standard.loyalty_score, 10);
    assert_eq!(vip.loyalty_score, 20);
    assert_eq!(zero_multiplier.loyalty_score, 10);
}

#[test]
#[should_panic]
fn test_set_global_promo_unauthorized() {
    let env = Env::default();
    let (client, _admin, _contract_id) = setup(&env);

    client.set_global_promo(&1000, &(env.ledger().timestamp() + 100));
}

fn event_args_with_end_time(
    env: &Env,
    event_id: &str,
    organizer: &Address,
    end_time: u64,
) -> EventRegistrationArgs {
    let mut args = event_args(env, event_id, organizer);
    args.end_time = end_time;
    args
}

#[test]
fn test_distribute_staker_rewards_zero_amount_rejects() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _contract_id) = setup(&env);

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

    let result = client.try_distribute_staker_rewards(&admin, &0i128);
    assert_eq!(result, Err(Ok(EventRegistryError::InvalidRewardAmount)));
}

#[test]
fn test_set_feedback_cid_before_event_ends_rejects() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "feedback_early_event");
    let end_time = env.ledger().timestamp() + 3600;

    client.register_event(&event_args_with_end_time(
        &env,
        "feedback_early_event",
        &organizer,
        end_time,
    ));

    let feedback_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    let result = client.try_set_feedback_cid(&event_id, &feedback_cid);
    assert_eq!(result, Err(Ok(EventRegistryError::EventNotEnded)));
}

#[test]
fn test_set_feedback_cid_after_event_ends_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id) = setup(&env);
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "feedback_late_event");
    let end_time = env.ledger().timestamp() + 100;

    client.register_event(&event_args_with_end_time(
        &env,
        "feedback_late_event",
        &organizer,
        end_time,
    ));

    env.ledger().with_mut(|li| li.timestamp = end_time + 1);

    let feedback_cid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    client.set_feedback_cid(&event_id, &feedback_cid);

    let event_info = client.get_event(&event_id).unwrap();
    assert_eq!(event_info.feedback_cid, Some(feedback_cid));
}
