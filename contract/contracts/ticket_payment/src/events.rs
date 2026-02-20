use soroban_sdk::{contractevent, Address, BytesN};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializationEvent {
    pub usdc_token: Address,
    pub platform_wallet: Address,
    pub event_registry: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractUpgraded {
    pub old_wasm_hash: BytesN<32>,
    pub new_wasm_hash: BytesN<32>,
}
