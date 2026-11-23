use super::*;
use crate::error::EventRegistryError;
use crate::types::{EventRegistrationArgs, EventStatus};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Map, String,
};

/// Registers a contract, initializes it, and registers one event with
/// `start_time` as its current start. Returns the client, the event id and
/// the organizer.
fn setup_event(env: &Env, start_time: u64) -> (EventRegistryClient<'_>, String, Address) {
    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let platform_wallet = Address::generate(env);
    let usdc_token = Address::generate(env);
    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    let organizer = Address::generate(env);
    let event_id = String::from_str(env, "postpone_event");
    client.register_event(&EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(env, "Postpone Test"),
        organizer_address: organizer.clone(),
        payment_address: Address::generate(env),
        metadata_cid: String::from_str(
            env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        ),
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
        category_ids: None,
        start_time,
        is_private: false,
        end_time: 0,
        transfer_lock_duration: 0,
        accepted_tokens: soroban_sdk::Vec::new(env),
        use_global_whitelist: true,
        referral_rate_bps: None,
    });

    (client, event_id, organizer)
}

#[test]
fn test_postpone_happy_path_allows_grace_equal_to_new_start() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, event_id, _organizer) = setup_event(&env, 2_000);

    // New start 3_000 is later than 2_000, and the grace window closes exactly
    // at the new start, which the boundary rule allows.
    client.postpone_event(&event_id, &3_000u64, &3_000u64);

    let info = client.get_event(&event_id).unwrap();
    assert!(info.is_postponed);
    assert_eq!(info.start_time, 3_000);
    assert_eq!(info.grace_period_end, 3_000);
}

#[test]
fn test_postpone_cancelled_event_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, event_id, _organizer) = setup_event(&env, 2_000);

    client.cancel_event(&event_id, &None);

    let res = client.try_postpone_event(&event_id, &3_000u64, &2_500u64);
    assert_eq!(res, Err(Ok(EventRegistryError::EventCancelled)));
    assert!(!client.get_event(&event_id).unwrap().is_postponed);
}

#[test]
fn test_postpone_inactive_event_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, event_id, _organizer) = setup_event(&env, 2_000);

    // Deactivate the event the way suspension does: clear is_active, keep status.
    env.as_contract(&client.address, || {
        let mut info = storage::get_event(&env, event_id.clone()).unwrap();
        info.is_active = false;
        storage::store_event(&env, info);
    });

    let res = client.try_postpone_event(&event_id, &3_000u64, &2_500u64);
    assert_eq!(res, Err(Ok(EventRegistryError::EventInactive)));
    assert!(!client.get_event(&event_id).unwrap().is_postponed);
}

#[test]
fn test_postpone_status_inactive_event_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, event_id, _organizer) = setup_event(&env, 2_000);

    env.as_contract(&client.address, || {
        let mut info = storage::get_event(&env, event_id.clone()).unwrap();
        info.status = EventStatus::Inactive;
        info.is_active = true;
        storage::store_event(&env, info);
    });

    let res = client.try_postpone_event(&event_id, &3_000u64, &2_500u64);
    assert_eq!(res, Err(Ok(EventRegistryError::EventInactive)));
}

#[test]
fn test_postpone_earlier_start_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, event_id, _organizer) = setup_event(&env, 5_000);

    // 4_000 is in the future but earlier than the current start of 5_000.
    let res = client.try_postpone_event(&event_id, &4_000u64, &3_500u64);
    assert_eq!(res, Err(Ok(EventRegistryError::InvalidDeadline)));
    assert!(!client.get_event(&event_id).unwrap().is_postponed);
}

#[test]
fn test_postpone_same_start_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, event_id, _organizer) = setup_event(&env, 5_000);

    // Equal to the current start is not a move forward.
    let res = client.try_postpone_event(&event_id, &5_000u64, &4_500u64);
    assert_eq!(res, Err(Ok(EventRegistryError::InvalidDeadline)));
}

#[test]
fn test_postpone_grace_after_new_start_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, event_id, _organizer) = setup_event(&env, 2_000);

    // Grace window (4_000) outlives the rescheduled start (3_000).
    let res = client.try_postpone_event(&event_id, &3_000u64, &4_000u64);
    assert_eq!(res, Err(Ok(EventRegistryError::InvalidGracePeriod)));
    assert!(!client.get_event(&event_id).unwrap().is_postponed);
}
