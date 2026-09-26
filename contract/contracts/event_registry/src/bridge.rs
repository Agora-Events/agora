use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
pub struct BridgeConfig {
    pub relayer_pubkeys: Vec<Address>,
    pub threshold: u32,
    pub nonce: u64,
}

pub fn set_bridge_config(env: &Env, config: &BridgeConfig) {
    env.storage().instance().set(&"BridgeConfig", config);
}

pub fn get_bridge_config(env: &Env) -> Option<BridgeConfig> {
    env.storage().instance().get(&"BridgeConfig")
}
