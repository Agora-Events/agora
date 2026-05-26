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

// ─────────────────────────────────────────────────────────────────────────────
// New test cases required by issue C-58
// ─────────────────────────────────────────────────────────────────────────────

/// Cancel a proposal as the proposer, then try to cancel it again.
/// The second cancel must return ProposalAlreadyCancelled (error code #49).
#[test]
fn test_cancel_proposal_by_proposer() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // admin1 creates a proposal
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);

    // admin1 (the proposer) cancels it — must succeed
    client.cancel_proposal(&admin1, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.cancelled, "proposal should be marked cancelled");
    assert!(!proposal.executed, "proposal should not be executed");

    // Cancelling again must return ProposalAlreadyCancelled (#49)
    let result = client.try_cancel_proposal(&admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(crate::error::EventRegistryError::ProposalAlreadyCancelled)),
        "second cancel should return ProposalAlreadyCancelled"
    );
}

/// Create, vote on, and execute a proposal, then try to cancel it.
/// Must return ProposalAlreadyExecuted (error code #38).
#[test]
fn test_cancel_already_executed_proposal() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Create and immediately execute a proposal (threshold = 1, so one approval suffices)
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed, "proposal should be executed before cancel attempt");

    // Trying to cancel an already-executed proposal must return ProposalAlreadyExecuted (#38)
    let result = client.try_cancel_proposal(&admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(crate::error::EventRegistryError::ProposalAlreadyExecuted)),
        "cancelling an executed proposal should return ProposalAlreadyExecuted"
    );
}

/// admin1 creates a proposal; admin2 (non-proposer) tries to cancel it.
/// Must return Unauthorized (error code #3).
#[test]
fn test_cancel_proposal_by_non_proposer() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Add admin2 so they are a valid admin but not the proposer of the next proposal
    let add_admin2_proposal = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &add_admin2_proposal);

    // admin1 creates a new proposal
    let proposal_id = client.propose_set_platform_fee(&admin1, &800, &0);

    // admin2 (not the proposer) tries to cancel — must return Unauthorized (#3)
    let result = client.try_cancel_proposal(&admin2, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(crate::error::EventRegistryError::Unauthorized)),
        "non-proposer cancelling a proposal should return Unauthorized"
    );

    // The proposal must still be active (not cancelled)
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(!proposal.cancelled, "proposal should not be cancelled after unauthorized attempt");
}

/// Cancel a proposal, then try to vote on it.
/// Must return ProposalAlreadyCancelled (error code #49).
#[test]
fn test_vote_on_cancelled_proposal() {
    let (env, client, admin1, admin2) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&admin1, &platform_wallet, &500, &usdc_token);

    // Add admin2 so there is a second voter
    let new_admins = soroban_sdk::vec![&env, admin1.clone(), admin2.clone()];
    client.set_multisig_config(&admin1, &new_admins, &2);

    // admin1 creates a proposal that requires 2 approvals
    let proposal_id = client.propose_set_platform_fee(&admin1, &1000, &0);

    // admin1 cancels the proposal
    client.cancel_proposal(&admin1, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.cancelled, "proposal should be cancelled");

    // admin2 tries to approve the cancelled proposal — must return ProposalAlreadyCancelled (#49)
    let result = client.try_approve_proposal(&admin2, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(crate::error::EventRegistryError::ProposalAlreadyCancelled)),
        "voting on a cancelled proposal should return ProposalAlreadyCancelled"
    );
}
