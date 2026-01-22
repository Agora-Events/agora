use soroban_sdk::{contracttype, Address, Env};

/// Storage keys for the contract
#[contracttype]
pub enum DataKey {
    PlatformFee, // u32 - Platform fee percentage (0-100)
    Admin,       // Address - Contract administrator
}

/// Checks if the platform fee has been set in storage
pub fn has_platform_fee(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::PlatformFee)
}

/// Retrieves the current platform fee from storage
pub fn get_platform_fee(e: &Env) -> u32 {
    e.storage().instance().get(&DataKey::PlatformFee).unwrap()
}

/// Updates the platform fee in storage
pub fn set_platform_fee(e: &Env, fee: u32) {
    e.storage().instance().set(&DataKey::PlatformFee, &fee);
}

/// Checks if the admin address has been set in storage
pub fn has_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

/// Retrieves the admin address from storage
pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

/// Sets the admin address in storage
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}
