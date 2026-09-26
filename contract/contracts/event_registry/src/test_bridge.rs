#![cfg(test)]

use crate::bridge::{BridgeConfig, set_bridge_config};
use crate::error::EventRegistryError;
use crate::EventRegistry;
use crate::types::{EventRegistrationArgs, TicketTier};
use soroban_sdk::{testutils::Address as _, Address, Env, Vec, BytesN, String, Map, Bytes};

#[test]
fn test_mint_cross_chain_ticket_replay_protection() {
    let env = Env::default();
    env.mock_all_auths(); // Bypass auth for tests
    
    let contract_id = env.register(EventRegistry, ());
    let client = crate::EventRegistryClient::new(&env, &contract_id);

    // Initialize
    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc = Address::generate(&env);
    client.initialize(&admin, &platform_wallet, &0, &usdc);

    // Register an event
    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event1");
    let tier_id = String::from_str(&env, "tier1");

    let mut tiers = Map::new(&env);
    tiers.set(tier_id.clone(), TicketTier {
        name: String::from_str(&env, "General"),
        price: 1000,
        tier_limit: 100,
        current_sold: 0,
        is_refundable: false,
        auction_config: Vec::new(&env),
        loyalty_multiplier: 1,
        max_per_user: 10,
    });

    let args = EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(&env, "Event 1"),
        organizer_address: organizer.clone(),
        payment_address: Address::generate(&env),
        metadata_cid: String::from_str(&env, "QmYwAPJQK5CS5zGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
        max_supply: 100,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        milestone_plan: None,
        tags: None,
        category_ids: None,
        start_time: 0,
        is_private: false,
        end_time: 0,
        transfer_lock_duration: 0,
        accepted_tokens: Vec::new(&env),
        use_global_whitelist: true,
        banner_cid: None,
        referral_rate_bps: None,
        min_sales_target: None,
        target_deadline: None,
    };
    client.register_event(&args);

    // Setup bridge config
    let relayer = Address::generate(&env);
    let mut relayer_pubkeys = Vec::new(&env);
    relayer_pubkeys.push_back(relayer);
    let bridge_config = BridgeConfig {
        relayer_pubkeys,
        threshold: 1,
        nonce: 0,
    };
    
    // Wrap storage calls
    env.as_contract(&contract_id, || {
        set_bridge_config(&env, &bridge_config);
    });

    // Valid mock proof
    let origin_chain_id = 1u64;
    let origin_tx_hash = BytesN::from_array(&env, &[1u8; 32]);
    let recipient = Address::generate(&env);
    let quantity = 1u32;
    
    // Provide a dummy signature to satisfy threshold = 1
    let mut signatures = Vec::new(&env);
    signatures.push_back(Bytes::from_array(&env, &[0u8; 32])); 

    // First attempt should work
    let res = client.try_mint_cross_chain_ticket(
        &origin_chain_id,
        &origin_tx_hash,
        &recipient,
        &tier_id,
        &signatures,
        &event_id,
        &quantity,
    );
    assert!(res.is_ok(), "Minting failed: {:?}", res);

    // Second attempt (replay) should fail
    let res_replay = client.try_mint_cross_chain_ticket(
        &origin_chain_id,
        &origin_tx_hash,
        &recipient,
        &tier_id,
        &signatures,
        &event_id,
        &quantity,
    );
    assert_eq!(res_replay, Err(Ok(EventRegistryError::ReplayAttackDetected)));
}
