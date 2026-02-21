#[cfg(test)]
mod custom_fee_tests {
    use super::*;

    #[test]
    fn test_set_custom_event_fee() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let platform_wallet = Address::generate(&env);
        let contract_id = env.register_contract(None, EventRegistry);
        let client = EventRegistryClient::new(&env, &contract_id);

        // Initialize
        client.initialize(&admin, &platform_wallet, &500); // 5% default

        // Register event
        let organizer = Address::generate(&env);
        let event_id = String::from_str(&env, "charity-event-001");
        let payment_addr = Address::generate(&env);
        let metadata = String::from_str(&env, "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi");
        let tiers = Map::new(&env);
        
        client.register_event(&event_id, &organizer, &payment_addr, &metadata, &0, &tiers);

        // Set custom fee for charity event (0% fee)
        client.set_custom_event_fee(&event_id, &0);

        // Verify custom fee is applied
        let payment_info = client.get_event_payment_info(&event_id).unwrap();
        assert_eq!(payment_info.platform_fee_percent, 0);

        // Set custom fee for high-volume partner (2% fee)
        let partner_event = String::from_str(&env, "partner-event-001");
        client.register_event(&partner_event, &organizer, &payment_addr, &metadata, &0, &tiers);
        client.set_custom_event_fee(&partner_event, &200); // 2%

        let partner_info = client.get_event_payment_info(&partner_event).unwrap();
        assert_eq!(partner_info.platform_fee_percent, 200);
    }

    #[test]
    #[should_panic(expected = "InvalidFeePercent")]
    fn test_set_custom_fee_exceeds_max() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let platform_wallet = Address::generate(&env);
        let contract_id = env.register_contract(None, EventRegistry);
        let client = EventRegistryClient::new(&env, &contract_id);

        client.initialize(&admin, &platform_wallet, &500);

        let organizer = Address::generate(&env);
        let event_id = String::from_str(&env, "test-event");
        let payment_addr = Address::generate(&env);
        let metadata = String::from_str(&env, "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi");
        let tiers = Map::new(&env);
        
        client.register_event(&event_id, &organizer, &payment_addr, &metadata, &0, &tiers);

        // Should fail - fee > 10000 bps (100%)
        client.set_custom_event_fee(&event_id, &10001);
    }

    #[test]
    fn test_custom_fee_in_payment_calculation() {
        // This test would verify that the payment contract
        // correctly uses the custom fee when processing payments
        // Integration test between EventRegistry and TicketPayment contracts
    }
}
