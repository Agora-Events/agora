#![cfg(test)]

use super::*;
use soroban_sdk::{Env, Address, testutils::Address as _};

#[test]
fn test_initialize_platform_fee() {
    let env = Env::default();
    let contract_id = env.register(EventRegistryContract, ());
    let client = EventRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize_platform_fee(&admin, &5);

    assert_eq!(client.get_platform_fee(), 5);
    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic(expected = "Platform fee already initialized")]
fn test_double_initialization_fails() {
    let env = Env::default();
    let contract_id = env.register(EventRegistryContract, ());
    let client = EventRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize_platform_fee(&admin, &5);
    client.initialize_platform_fee(&admin, &10); // Should panic
}

#[test]
#[should_panic(expected = "Fee percent must be between 0 and 100")]
fn test_initialization_invalid_fee() {
    let env = Env::default();
    let contract_id = env.register(EventRegistryContract, ());
    let client = EventRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize_platform_fee(&admin, &101); // Should panic
}

#[test]
fn test_set_platform_fee() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(EventRegistryContract, ());
    let client = EventRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize_platform_fee(&admin, &5);

    client.set_platform_fee(&10);

    assert_eq!(client.get_platform_fee(), 10);
}

#[test]
#[should_panic(expected = "Fee percent must be between 0 and 100")]
fn test_set_platform_fee_invalid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(EventRegistryContract, ());
    let client = EventRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize_platform_fee(&admin, &5);

    client.set_platform_fee(&101); // Should panic
}

#[test]
#[should_panic] // Authentication failure
fn test_set_platform_fee_unauthorized() {
    let env = Env::default();
    
    let contract_id = env.register(EventRegistryContract, ());
    let client = EventRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize_platform_fee(&admin, &5);

    // This will fail because no auth is mocked/provided for the admin address stored in the contract
    client.set_platform_fee(&10);
}
