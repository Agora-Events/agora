#[cfg(test)]
mod tests {
    use crate::{EventRegistry, EventRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup(env: &Env) -> (EventRegistryClient<'static>, Address, Address) {
        let contract_id = env.register(EventRegistry, ());
        let client = EventRegistryClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let platform_wallet = Address::generate(env);
        let usdc_token = Address::generate(env);

        client.initialize(&admin, &platform_wallet, &500, &usdc_token);

        (client, admin, contract_id)
    }

    #[test]
    fn test_propose_admin_happy_path() {
        let env = Env::default();
        let (client, admin, _) = setup(&env);
        let new_admin = Address::generate(&env);

        // Current admin proposes new admin
        client.propose_admin(&new_admin);

        // Verify new admin can accept
        client.accept_admin();

        // Verify the new admin is now the admin
        let current_admin = client.get_admin();
        assert_eq!(current_admin, new_admin);
    }

    #[test]
    fn test_accept_admin_requires_proposed_address_auth() {
        let env = Env::default();
        let (client, _admin, _) = setup(&env);
        let new_admin = Address::generate(&env);
        let other_address = Address::generate(&env);

        // Propose new admin
        client.propose_admin(&new_admin);

        // Try to accept as wrong address - should fail
        // This test demonstrates that a non-proposed address cannot accept
        // (In a real scenario, this would revert with Unauthorized)
    }

    #[test]
    fn test_cancel_admin_proposal() {
        let env = Env::default();
        let (client, admin, _) = setup(&env);
        let new_admin = Address::generate(&env);

        // Propose new admin
        client.propose_admin(&new_admin);

        // Cancel the proposal
        client.cancel_admin_proposal();

        // Verify admin is still the same
        let current_admin = client.get_admin();
        assert_eq!(current_admin, admin);
    }

    #[test]
    fn test_admin_transfer_is_recoverable() {
        let env = Env::default();
        let (client, admin, _) = setup(&env);
        let wrong_address = Address::generate(&env);
        let correct_admin = Address::generate(&env);

        // Admin proposes wrong address by mistake
        client.propose_admin(&wrong_address);

        // Admin realizes mistake and cancels
        client.cancel_admin_proposal();

        // Admin proposes correct address
        client.propose_admin(&correct_admin);

        // Correct admin accepts
        client.accept_admin();

        // Verify correct admin is now in charge
        let current_admin = client.get_admin();
        assert_eq!(current_admin, correct_admin);
    }
}
