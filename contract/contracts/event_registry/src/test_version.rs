#[cfg(test)]
mod tests {
    use crate::{EventRegistry, EventRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_version_is_non_empty() {
        let env = Env::default();
        let contract_id = env.register(EventRegistry, ());
        let client = EventRegistryClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let platform_wallet = Address::generate(&env);
        let usdc_token = Address::generate(&env);

        client.initialize(&admin, &platform_wallet, &500, &usdc_token);

        let version = client.version();
        assert!(!version.is_empty(), "Version should not be empty");
        assert!(
            version.len() > 0,
            "Version string must contain at least one character"
        );
    }
}
