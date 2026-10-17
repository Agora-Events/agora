//! Unit tests for `get_event_capacity` (#1564).
//!
//! `get_event_capacity` was added alongside these tests because #1560 (the
//! issue that was supposed to introduce it) had not yet landed on `main` at
//! the time this PR was opened — see the comment on the function itself in
//! `lib.rs`. These tests exercise the real contract entry point, not a stub.
//!
//! Also covers tier price validation (#1559) and the `MetadataUpdated` event
//! emitted by `update_metadata` (#1561).

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

    /// Registration args whose single tier is priced at `price` stroops.
    fn priced_event_args(
        env: &Env,
        event_id: &str,
        organizer: &Address,
        price: i128,
    ) -> EventRegistrationArgs {
        let mut args = event_args(env, event_id, organizer, 100);
        let tier_id = String::from_str(env, "tier_00");
        let mut tier = args.tiers.get(tier_id.clone()).unwrap();
        tier.price = price;
        args.tiers.set(tier_id, tier);
        args
    }

    #[test]
    fn register_event_rejects_negative_tier_price() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);
        let organizer = Address::generate(&env);

        let result = client.try_register_event(&priced_event_args(&env, "neg", &organizer, -1));
        assert_eq!(result, Err(Ok(EventRegistryError::InvalidPrice)));
        assert!(client.get_event(&String::from_str(&env, "neg")).is_none());
    }

    #[test]
    fn register_event_accepts_one_stroop_and_free_tiers() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);
        let organizer = Address::generate(&env);

        // 1 stroop is the smallest paid price; 0 remains valid for free tickets.
        client.register_event(&priced_event_args(&env, "one_stroop", &organizer, 1));
        client.register_event(&priced_event_args(&env, "free", &organizer, 0));

        assert!(client.get_event(&String::from_str(&env, "one_stroop")).is_some());
        assert!(client.get_event(&String::from_str(&env, "free")).is_some());
    }

    #[test]
    fn add_tier_rejects_negative_price() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);
        let organizer = Address::generate(&env);
        let event_id = String::from_str(&env, "concert_1");

        let args = event_args(&env, "concert_1", &organizer, 0);
        let mut tier = args.tiers.get(String::from_str(&env, "tier_00")).unwrap();
        client.register_event(&args);

        tier.price = -500;
        let result = client.try_add_tier(&event_id, &String::from_str(&env, "vip"), &tier);
        assert_eq!(result, Err(Ok(EventRegistryError::InvalidPrice)));
    }

    #[test]
    fn update_metadata_emits_metadata_updated_event() {
        use crate::events::MetadataUpdatedEvent;
        use crate::topics::AgoraEvent;
        use soroban_sdk::testutils::Events as _;
        use soroban_sdk::TryIntoVal;

        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);
        let organizer = Address::generate(&env);
        let event_id = String::from_str(&env, "concert_1");

        client.register_event(&event_args(&env, "concert_1", &organizer, 100));

        let new_metadata_cid = String::from_str(
            &env,
            "bafkreifh22222222222222222222222222222222222222222222222222",
        );
        client.update_metadata(&event_id, &new_metadata_cid);

        let events = env.events().all();
        assert_eq!(events.len(), 1, "expected a single MetadataUpdated event");

        let (emitter, topics, data) = events.get(0).unwrap();
        assert_eq!(emitter, client.address);

        let topic: AgoraEvent = topics.get(0).unwrap().try_into_val(&env).unwrap();
        assert_eq!(topic, AgoraEvent::MetadataUpdated);

        let payload: MetadataUpdatedEvent = data.try_into_val(&env).unwrap();
        assert_eq!(payload.event_id, event_id);
        assert_eq!(payload.old_metadata_cid, metadata_cid(&env));
        assert_eq!(payload.new_metadata_cid, new_metadata_cid);
        assert_eq!(payload.updated_by, organizer);
    }

    #[test]
    fn update_metadata_with_unchanged_cid_emits_nothing() {
        use soroban_sdk::testutils::Events as _;

        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup(&env);
        let organizer = Address::generate(&env);
        let event_id = String::from_str(&env, "concert_1");

        client.register_event(&event_args(&env, "concert_1", &organizer, 100));
        client.update_metadata(&event_id, &metadata_cid(&env));

        assert_eq!(env.events().all().len(), 0);
    }
}
