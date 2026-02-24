use crate::{EventRegistry, EventRegistryClient};
use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Env};

fn create_test_env() -> (Env, EventRegistryClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);

    (env, client, admin1, admin2, admin3)
}

#[test]
fn test_initialize_with_multisig() {
    let (env, client, admin, _, _) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500);

    // Verify multi-sig config was created
    let config = client.get_multisig_config();
    assert_eq!(config.admins.len(), 1);
    assert_eq!(config.admins.get(0).unwrap(), admin);
    assert_eq!(config.threshold, 1);
}

#[test]
fn test_create_proposal_add_admin() {
    let (env, client, admin1, admin2, _) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Create proposal to add admin2
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);

    // Verify proposal was created
    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.proposal_id, proposal_id);
    assert_eq!(proposal.proposer, admin1);
    assert_eq!(proposal.approvals.len(), 1); // Proposer auto-approves
    assert!(!proposal.executed);
}

#[test]
fn test_execute_proposal_single_admin() {
    let (env, client, admin1, admin2, _) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Create and execute proposal to add admin2 (threshold = 1, so auto-approved)
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Verify admin2 was added
    assert!(client.is_admin(&admin2));

    // Verify proposal was marked as executed
    let proposal = client.get_proposal(&proposal_id);
    assert!(proposal.executed);
}

#[test]
fn test_multisig_workflow_add_admin() {
    let (env, client, admin1, admin2, admin3) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Add admin2 (threshold = 1)
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Set threshold to 2
    let proposal_id = client.propose_set_threshold(&admin1, &2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    let config = client.get_multisig_config();
    assert_eq!(config.threshold, 2);

    // Now try to add admin3 - requires 2 approvals
    let proposal_id = client.propose_add_admin(&admin1, &admin3, &0);

    // Try to execute with only 1 approval - should fail
    let result = client.try_execute_proposal(&admin1, &proposal_id);
    assert!(result.is_err());

    // Admin2 approves
    client.approve_proposal(&admin2, &proposal_id);

    // Now execute should succeed
    client.execute_proposal(&admin1, &proposal_id);

    // Verify admin3 was added
    assert!(client.is_admin(&admin3));
}

#[test]
fn test_propose_set_platform_wallet() {
    let (env, client, admin1, admin2, _) = create_test_env();
    let platform_wallet = Address::generate(&env);
    let new_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Add admin2 and set threshold to 2
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    let proposal_id = client.propose_set_threshold(&admin1, &2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Propose to change platform wallet
    let proposal_id = client.propose_set_platform_wallet(&admin1, &new_wallet, &0);

    // Admin2 approves
    client.approve_proposal(&admin2, &proposal_id);

    // Execute
    client.execute_proposal(&admin1, &proposal_id);

    // Verify wallet was changed
    let current_wallet = client.get_platform_wallet();
    assert_eq!(current_wallet, new_wallet);
}

#[test]
fn test_remove_admin_with_multisig() {
    let (env, client, admin1, admin2, admin3) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Add admin2 and admin3
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    let proposal_id = client.propose_add_admin(&admin1, &admin3, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Set threshold to 2
    let proposal_id = client.propose_set_threshold(&admin1, &2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Propose to remove admin3
    let proposal_id = client.propose_remove_admin(&admin1, &admin3, &0);

    // Admin2 approves
    client.approve_proposal(&admin2, &proposal_id);

    // Execute
    client.execute_proposal(&admin1, &proposal_id);

    // Verify admin3 was removed
    assert!(!client.is_admin(&admin3));

    // Verify threshold was adjusted if needed
    let config = client.get_multisig_config();
    assert_eq!(config.admins.len(), 2);
}

#[test]
#[should_panic(expected = "CannotRemoveLastAdmin")]
fn test_cannot_remove_last_admin() {
    let (env, client, admin1, _, _) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Try to remove the only admin - should fail
    client.propose_remove_admin(&admin1, &admin1, &0);
}

#[test]
#[should_panic(expected = "AdminAlreadyExists")]
fn test_cannot_add_duplicate_admin() {
    let (env, client, admin1, _, _) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Try to add admin1 again - should fail
    client.propose_add_admin(&admin1, &admin1, &0);
}

#[test]
#[should_panic(expected = "AlreadyApproved")]
fn test_cannot_approve_twice() {
    let (env, client, admin1, admin2, admin3) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Add admin2
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Set threshold to 2
    let proposal_id = client.propose_set_threshold(&admin1, &2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Create proposal to add admin3
    let proposal_id = client.propose_add_admin(&admin1, &admin3, &0);

    // Admin1 tries to approve again (already auto-approved as proposer)
    client.approve_proposal(&admin1, &proposal_id);
}

#[test]
#[should_panic(expected = "ProposalAlreadyExecuted")]
fn test_cannot_execute_twice() {
    let (env, client, admin1, admin2, _) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Create and execute proposal
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Try to execute again - should fail
    client.execute_proposal(&admin1, &proposal_id);
}

#[test]
#[should_panic(expected = "InvalidThreshold")]
fn test_invalid_threshold_too_high() {
    let (env, client, admin1, _, _) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Try to set threshold higher than admin count - should fail
    client.propose_set_threshold(&admin1, &5, &0);
}

#[test]
#[should_panic(expected = "InvalidThreshold")]
fn test_invalid_threshold_zero() {
    let (env, client, admin1, _, _) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Try to set threshold to 0 - should fail
    client.propose_set_threshold(&admin1, &0, &0);
}

#[test]
fn test_get_active_proposals() {
    let (env, client, admin1, admin2, admin3) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Create multiple proposals
    let proposal_id1 = client.propose_add_admin(&admin1, &admin2, &0);
    let proposal_id2 = client.propose_add_admin(&admin1, &admin3, &0);

    let active_proposals = client.get_active_proposals();
    assert_eq!(active_proposals.len(), 2);

    // Execute one proposal
    client.execute_proposal(&admin1, &proposal_id1);

    // Should have one less active proposal
    let active_proposals = client.get_active_proposals();
    assert_eq!(active_proposals.len(), 1);
    assert_eq!(active_proposals.get(0).unwrap(), proposal_id2);
}

#[test]
fn test_proposal_expiration() {
    let (env, client, admin1, admin2, admin3) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Add admin2 and set threshold to 2
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    let proposal_id = client.propose_set_threshold(&admin1, &2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Create proposal with short expiration (10 ledgers)
    let proposal_id = client.propose_add_admin(&admin1, &admin3, &10);

    // Advance ledger past expiration
    env.ledger().with_mut(|li| {
        li.timestamp += 100; // More than 10 ledgers * 5 seconds
    });

    // Try to approve - should fail due to expiration
    let result = client.try_approve_proposal(&admin2, &proposal_id);
    assert!(result.is_err());
}

#[test]
fn test_threshold_adjustment_on_admin_removal() {
    let (env, client, admin1, admin2, admin3) = create_test_env();
    let platform_wallet = Address::generate(&env);

    client.initialize(&admin1, &platform_wallet, &500);

    // Add admin2 and admin3
    let proposal_id = client.propose_add_admin(&admin1, &admin2, &0);
    client.execute_proposal(&admin1, &proposal_id);

    let proposal_id = client.propose_add_admin(&admin1, &admin3, &0);
    client.execute_proposal(&admin1, &proposal_id);

    // Set threshold to 3 (all admins required)
    let proposal_id = client.propose_set_threshold(&admin1, &3, &0);
    client.execute_proposal(&admin1, &proposal_id);

    let config = client.get_multisig_config();
    assert_eq!(config.threshold, 3);

    // Propose to remove admin3 (requires all 3 approvals)
    let proposal_id = client.propose_remove_admin(&admin1, &admin3, &0);
    client.approve_proposal(&admin2, &proposal_id);
    client.approve_proposal(&admin3, &proposal_id);
    client.execute_proposal(&admin1, &proposal_id);

    // Threshold should be adjusted to 2 (can't be higher than admin count)
    let config = client.get_multisig_config();
    assert_eq!(config.admins.len(), 2);
    assert_eq!(config.threshold, 2);
}
