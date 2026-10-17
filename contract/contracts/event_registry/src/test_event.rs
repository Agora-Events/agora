//! Unit tests for `get_event_capacity` (#1564).
//!
//! `get_event_capacity` was added alongside these tests because #1560 (the
//! issue that was supposed to introduce it) had not yet landed on `main` at
//! the time this PR was opened — see the comment on the function itself in
//! `lib.rs`. These tests exercise the real contract entry point, not a stub.

#[cfg(test)]
mod tests {
    use crate::error::EventRegistryError;
    use crate::types::{EventRegistrationArgs, TicketTier};
    use crate::{EventRegistry, EventRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

    fn setup(env: &Env) -> (EventRegistryClient<'static>, Address) {
        let contract_id = env.register(EventRegistry, ());
        let client = EventRegistryClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let platform_wallet = Address::generate(env);
        let usdc_token = Address::generate(env);

        client.initialize(&admin, &platform_wallet, &500, &usdc_token);

        (client, admin)
    }

    fn metadata_cid(env: &Env) -> String {
        String::from_str(
            env,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        )
    }

    /// Builds registration args for a single-tier event with the given
    /// `max_supply` (the value `get_event_capacity` should report back).
    fn event_args(
        env: &Env,
        event_id: &str,
        organizer: &Address,
        max_supply: i128,
    ) -> EventRegistrationArgs {
        let mut tiers = Map::new(env);
        tiers.set(
            String::from_str(env, "tier_00"),
            TicketTier {
                name: String::from_str(env, "General"),
                price: 1000,
                tier_limit: max_supply as u32,
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

    #[test]
    fn get_event_capacity_returns_max_supply_for_existing_event() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);
        let organizer = Address::generate(&env);

        client.register_event(&event_args(&env, "concert_1", &organizer, 500));

        let capacity = client.get_event_capacity(&String::from_str(&env, "concert_1"));
        assert_eq!(capacity, 500);
    }

    #[test]
    fn get_event_capacity_reflects_each_events_own_max_supply() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);
        let organizer = Address::generate(&env);

        client.register_event(&event_args(&env, "small_event", &organizer, 10));
        client.register_event(&event_args(&env, "large_event", &organizer, 10_000));

        assert_eq!(
            client.get_event_capacity(&String::from_str(&env, "small_event")),
            10
        );
        assert_eq!(
            client.get_event_capacity(&String::from_str(&env, "large_event")),
            10_000
        );
    }

    #[test]
    fn get_event_capacity_supports_unlimited_capacity_sentinel() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);
        let organizer = Address::generate(&env);

        // max_supply = 0 means "unlimited" per the EventInfo field doc.
        client.register_event(&event_args(&env, "unlimited_event", &organizer, 0));

        let capacity = client.get_event_capacity(&String::from_str(&env, "unlimited_event"));
        assert_eq!(capacity, 0);
    }

    #[test]
    fn get_event_capacity_errors_for_unknown_event() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);

        let result = client.try_get_event_capacity(&String::from_str(&env, "does_not_exist"));
        assert_eq!(
            result,
            Err(Ok(EventRegistryError::EventNotFound))
        );
    }
}
