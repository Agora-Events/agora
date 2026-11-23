//! Tests for `get_events_batch` that mirror its doc example (#1817): the
//! calls go through the generated `EventRegistryClient` with Soroban
//! `vec!`/`String`, exactly as integrators are told to write them.

#[cfg(test)]
mod tests {
    use crate::error::EventRegistryError;
    use crate::types::{EventRegistrationArgs, TicketTier};
    use crate::{EventRegistry, EventRegistryClient, MAX_BATCH_SIZE};
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String, Vec};

    fn setup(env: &Env) -> EventRegistryClient<'static> {
        let contract_id = env.register(EventRegistry, ());
        let client = EventRegistryClient::new(env, &contract_id);
        client.initialize(
            &Address::generate(env),
            &Address::generate(env),
            &500,
            &Address::generate(env),
        );
        client
    }

    fn register(env: &Env, client: &EventRegistryClient, event_id: &str) {
        let mut tiers = Map::new(env);
        tiers.set(
            String::from_str(env, "tier_00"),
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
        client.register_event(&EventRegistrationArgs {
            event_id: String::from_str(env, event_id),
            name: String::from_str(env, "Test Event"),
            organizer_address: Address::generate(env),
            payment_address: Address::generate(env),
            metadata_cid: String::from_str(
                env,
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
            category_ids: None,
            start_time: 0,
            is_private: false,
            end_time: 0,
            transfer_lock_duration: 0,
            accepted_tokens: Vec::new(env),
            use_global_whitelist: true,
            referral_rate_bps: None,
        });
    }

    #[test]
    fn doc_example_returns_results_in_input_order_with_none_for_unknown() {
        let env = Env::default();
        env.mock_all_auths();
        let client = setup(&env);
        register(&env, &client, "event1");
        register(&env, &client, "event2");

        let event_ids = vec![
            &env,
            String::from_str(&env, "event2"),
            String::from_str(&env, "missing"),
            String::from_str(&env, "event1"),
        ];
        let events = client.get_events_batch(&event_ids);

        assert_eq!(events.len(), 3);
        assert_eq!(
            events.get(0).unwrap().unwrap().event_id,
            String::from_str(&env, "event2")
        );
        assert!(events.get(1).unwrap().is_none());
        assert_eq!(
            events.get(2).unwrap().unwrap().event_id,
            String::from_str(&env, "event1")
        );
    }

    #[test]
    fn exactly_max_batch_size_is_accepted() {
        let env = Env::default();
        let client = setup(&env);
        let mut event_ids = Vec::new(&env);
        for _ in 0..MAX_BATCH_SIZE {
            event_ids.push_back(String::from_str(&env, "missing"));
        }
        let events = client.get_events_batch(&event_ids);
        assert_eq!(events.len(), MAX_BATCH_SIZE);
    }

    #[test]
    fn one_over_max_batch_size_returns_batch_size_exceeded() {
        let env = Env::default();
        let client = setup(&env);
        let mut event_ids = Vec::new(&env);
        for _ in 0..=MAX_BATCH_SIZE {
            event_ids.push_back(String::from_str(&env, "missing"));
        }
        assert_eq!(
            client.try_get_events_batch(&event_ids),
            Err(Ok(EventRegistryError::BatchSizeExceeded))
        );
    }
}
