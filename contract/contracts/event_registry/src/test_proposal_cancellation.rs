use crate::{EventRegistry, EventRegistryClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

fn create_test_env() -> (Env, EventRegistryClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    (env, client, admin1, admin2)
}

#[test]
fn test_cancel_proposal_success() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Create proposal
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);

    // Cancel proposal
    client.cancel_proposal(&admin1, &proposal_id);

    // Verify proposal is cancelled
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.cancelled);
    assert!(!proposal.executed);

    // Verify it's removed from active proposals
    let active_proposals = client.get_active_proposals();
    assert!(!active_proposals.contains(proposal_id));
}

#[test]
#[should_panic(expected = "Error(Contract, #49)")]
fn test_cannot_approve_cancelled_proposal() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Set threshold to 2 to require another approval
    let new_admins = soroban_sdk::vec![&env, admin1.clone(), admin2.clone()];
    client.set_multisig_config(&admin1, &new_admins, &2);

    // Create proposal
    let proposal_id = client.propose_set_platform_fee(&admin1, &1000, &0);

    // Cancel proposal
    client.cancel_proposal(&admin1, &proposal_id);

    // Admin2 tries to approve - should fail
    client.approve_proposal(&admin2, &proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #49)")]
fn test_cannot_execute_cancelled_proposal() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Create proposal
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);

    // Cancel proposal
    client.cancel_proposal(&admin1, &proposal_id);

    // Try to execute - should fail
    client.execute_proposal(&admin1, &proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_only_proposer_can_cancel() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Add admin2
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Admin1 creates another proposal
    let proposal_id2 = client.propose_set_platform_fee(&admin1, &1000, &0);

    // Admin2 tries to cancel Admin1's proposal - should fail
    client.cancel_proposal(&admin2, &proposal_id2);
}

#[test]
#[should_panic(expected = "Error(Contract, #49)")]
fn test_cannot_cancel_twice() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Create proposal
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);

    // Cancel once
    client.cancel_proposal(&admin1, &proposal_id);

    // Cancel again - should fail
    client.cancel_proposal(&admin1, &proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #38)")]
fn test_cannot_cancel_executed_proposal() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Create and execute proposal
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Try to cancel - should fail
    client.cancel_proposal(&admin1, &proposal_id);
}
