#![no_std]

use crate::types::EventInfo;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Vec};

pub mod storage;
pub mod types;

#[contract]
pub struct EventRegistry;

#[contractimpl]
impl EventRegistry {
    /// Initializes the contract with an admin address and initial platform fee.
    pub fn initialize(env: Env, admin: Address, platform_fee_percent: u32) {
        if storage::get_admin(&env).is_some() || storage::has_platform_fee(&env) {
            panic!("already initialized");
        }
        if platform_fee_percent > 100 {
            panic!("Fee percent must be between 0 and 100");
        }
        storage::set_admin(&env, &admin);
        storage::set_platform_fee(&env, platform_fee_percent);
    }

    /// Stores or updates an event.
    pub fn store_event(env: Env, event_info: EventInfo) {
        // In a real scenario, we would check authorization here.
        storage::store_event(&env, event_info);
    }

    /// Retrieves an event by its ID.
    pub fn get_event(env: Env, event_id: String) -> Option<EventInfo> {
        storage::get_event(&env, event_id)
    }

    /// Checks if an event exists.
    pub fn event_exists(env: Env, event_id: String) -> bool {
        storage::event_exists(&env, event_id)
    }

    /// Retrieves all event IDs for an organizer.
    pub fn get_organizer_events(env: Env, organizer: Address) -> Vec<String> {
        storage::get_organizer_events(&env, &organizer)
    }

    /// Updates the platform fee percentage. Only callable by the administrator.
    pub fn set_platform_fee(env: Env, new_fee_percent: u32) {
        let admin = storage::get_admin(&env).expect("Contract not initialized");
        admin.require_auth();

        if new_fee_percent > 100 {
            panic!("Fee percent must be between 0 and 100");
        }

        storage::set_platform_fee(&env, new_fee_percent);

        // Emit fee update event
        env.events().publish(
            (symbol_short!("fee_upd"),),
            new_fee_percent
        );
    }

    /// Returns the current platform fee percentage.
    pub fn get_platform_fee(env: Env) -> u32 {
        storage::get_platform_fee(&env)
    }

    /// Returns the current administrator address.
    pub fn get_admin(env: Env) -> Address {
        storage::get_admin(&env).expect("Contract not initialized")
    }
}

#[cfg(test)]
mod test;
