use crate::types::{DataKey, EventInfo};
use soroban_sdk::{Address, Env, String, Vec};

/// Sets the administrator address of the contract.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

/// Retrieves the administrator address of the contract.
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

/// Sets the global platform fee.
pub fn set_platform_fee(env: &Env, fee: u32) {
    env.storage().persistent().set(&DataKey::PlatformFee, &fee);
}

/// Retrieves the global platform fee.
pub fn get_platform_fee(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::PlatformFee)
        .unwrap_or(0)
}

/// Checks if the platform fee has been set.
pub fn has_platform_fee(env: &Env) -> bool {
    env.storage().persistent().has(&DataKey::PlatformFee)
}

/// Stores a new event or updates an existing one.
/// Also updates the organizer's list of events.
pub fn store_event(env: &Env, event_info: EventInfo) {
    let event_id = event_info.event_id.clone();
    let organizer = event_info.organizer_address.clone();

    // Store the event info using persistent storage
    env.storage()
        .persistent()
        .set(&DataKey::Event(event_id.clone()), &event_info);

    // Update organizer's event list
    let mut organizer_events: Vec<String> = get_organizer_events(env, &organizer);

    // Check if event_id is already in the list to avoid duplicates on updates
    let mut exists = false;
    for id in organizer_events.iter() {
        if id == event_id {
            exists = true;
            break;
        }
    }

    if !exists {
        organizer_events.push_back(event_id);
        env.storage()
            .persistent()
            .set(&DataKey::OrganizerEvents(organizer), &organizer_events);
    }
}

/// Retrieves event information by event_id.
pub fn get_event(env: &Env, event_id: String) -> Option<EventInfo> {
    env.storage().persistent().get(&DataKey::Event(event_id))
}

/// Checks if an event with the given event_id exists.
pub fn event_exists(env: &Env, event_id: String) -> bool {
    env.storage().persistent().has(&DataKey::Event(event_id))
}

/// Retrieves all event_ids associated with an organizer.
pub fn get_organizer_events(env: &Env, organizer: &Address) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&DataKey::OrganizerEvents(organizer.clone()))
        .unwrap_or_else(|| Vec::new(env))
}
