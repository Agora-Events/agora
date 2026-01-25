#![no_std]

use crate::types::{EventInfo, PaymentInfo};
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
        if platform_fee_percent > 10000 {
            panic!("Fee percent must be between 0 and 10000 (100%)");
        }
        storage::set_admin(&env, &admin);
        storage::set_platform_fee(&env, platform_fee_percent);
    }

    /// Register a new event with organizer authentication
    pub fn register_event(
        env: Env,
        event_id: String,
        organizer_address: Address,
        payment_address: Address,
    ) {
        // Verify organizer signature
        organizer_address.require_auth();

        // Check if event already exists
        if storage::event_exists(&env, event_id.clone()) {
            panic!("Event already exists");
        }

        // Get current platform fee
        let platform_fee_percent = storage::get_platform_fee(&env);

        // Create event info with current timestamp
        let event_info = EventInfo {
            event_id: event_id.clone(),
            organizer_address: organizer_address.clone(),
            payment_address: payment_address.clone(),
            platform_fee_percent,
            is_active: true,
            created_at: env.ledger().timestamp(),
        };

        // Store the event
        storage::store_event(&env, event_info);

        // Emit registration event
        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("event_reg"), event_id.clone()),
            (organizer_address, payment_address, platform_fee_percent),
        );
    }

    /// Get event payment information
    pub fn get_event_payment_info(env: Env, event_id: String) -> PaymentInfo {
        match storage::get_event(&env, event_id) {
            Some(event_info) => PaymentInfo {
                payment_address: event_info.payment_address,
                platform_fee_percent: event_info.platform_fee_percent,
            },
            None => panic!("Event not found"),
        }
    }

    /// Update event status (only by organizer)
    pub fn update_event_status(env: Env, event_id: String, is_active: bool) {
        match storage::get_event(&env, event_id.clone()) {
            Some(mut event_info) => {
                // Verify organizer signature
                event_info.organizer_address.require_auth();

                // Update status
                event_info.is_active = is_active;
                storage::store_event(&env, event_info);
            }
            None => panic!("Event not found"),
        }
    }

    /// Stores or updates an event (legacy function for backward compatibility).
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

        if new_fee_percent > 10000 {
            panic!("Fee percent must be between 0 and 10000 (100%)");
        }

        storage::set_platform_fee(&env, new_fee_percent);

        // Emit fee update event
        #[allow(deprecated)]
        env.events()
            .publish((symbol_short!("fee_upd"),), new_fee_percent);
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
