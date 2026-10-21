use soroban_sdk::{contracttype, Env, BytesN};

#[contracttype]
pub struct TeleportationInfo {
    pub origin_chain_id: u64,
    pub origin_tx_hash: BytesN<32>,
    pub processed: bool,
}

pub fn track_teleport(env: &Env, tx_hash: BytesN<32>) {
    env.storage().instance().set(&tx_hash, &true);
}

pub fn is_teleport_processed(env: &Env, tx_hash: BytesN<32>) -> bool {
    env.storage().instance().get(&tx_hash).unwrap_or(false)
}
