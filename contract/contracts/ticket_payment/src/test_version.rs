#[cfg(test)]
mod tests {
    use crate::{TicketPaymentContract, TicketPaymentContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_version_is_non_empty() {
        let env = Env::default();
        let contract_id = env.register(TicketPaymentContract, ());
        let client = TicketPaymentContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let usdc_token = Address::generate(&env);
        let platform_wallet = Address::generate(&env);
        let event_registry = Address::generate(&env);

        client.initialize(&admin, &usdc_token, &platform_wallet, &event_registry);

        let version = client.version();
        assert!(!version.is_empty(), "Version should not be empty");
        assert!(
            version.len() > 0,
            "Version string must contain at least one character"
        );
    }
}
