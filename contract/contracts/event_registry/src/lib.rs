#![no_std]
use crate::types::EventInfo;
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

pub mod storage;
pub mod types;

#[contract]
pub struct EventRegistry;

#[contractimpl]
impl EventRegistry {
    /// Initializes the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        if storage::get_admin(&env).is_some() {
            panic!("already initialized");
        }
        storage::set_admin(&env, &admin);
    }

    /// Stores or updates an event.
    pub fn store_event(env: Env, event_info: EventInfo) {
        // In a real scenario, we would check authorization here.
        // For this task, we focus on the storage implementation.
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
}

#[cfg(test)]
mod test;
