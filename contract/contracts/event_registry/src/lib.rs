#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, symbol_short};

mod storage;

#[contract]
pub struct EventRegistryContract;

#[contractimpl]
impl EventRegistryContract {
    /// Initializes the platform fee and sets the contract administrator.
    /// 
    /// # Parameters
    /// * `admin` - The address that will have administrator privileges.
    /// * `initial_fee_percent` - The initial platform fee percentage (0-100).
    /// 
    /// # Panics
    /// * If already initialized.
    /// * If `initial_fee_percent` is greater than 100.
    pub fn initialize_platform_fee(e: Env, admin: Address, initial_fee_percent: u32) {
        if storage::has_platform_fee(&e) || storage::has_admin(&e) {
            panic!("Platform fee already initialized");
        }
        if initial_fee_percent > 100 {
            panic!("Fee percent must be between 0 and 100");
        }
        
        storage::set_admin(&e, &admin);
        storage::set_platform_fee(&e, initial_fee_percent);
    }

    /// Updates the platform fee percentage. Only callable by the administrator.
    /// 
    /// # Parameters
    /// * `new_fee_percent` - The new platform fee percentage (0-100).
    /// 
    /// # Panics
    /// * If the caller is not the administrator.
    /// * If `new_fee_percent` is greater than 100.
    #[allow(deprecated)]
    pub fn set_platform_fee(e: Env, new_fee_percent: u32) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if new_fee_percent > 100 {
            panic!("Fee percent must be between 0 and 100");
        }

        storage::set_platform_fee(&e, new_fee_percent);

        // Emit fee update event
        e.events().publish(
            (symbol_short!("fee_upd"),),
            new_fee_percent
        );
    }

    /// Returns the current platform fee percentage.
    pub fn get_platform_fee(e: Env) -> u32 {
        storage::get_platform_fee(&e)
    }

    /// Returns the current administrator address.
    pub fn get_admin(e: Env) -> Address {
        storage::get_admin(&e)
    }
}

mod test;
