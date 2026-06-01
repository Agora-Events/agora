use crate::error::EventRegistryError;
use crate::storage;
use soroban_sdk::{Address, Env, String};

/// Verifies the caller is the admin and returns the admin address.
/// Returns an error if the contract is not initialized or the caller is not the admin.
pub fn require_admin(env: &Env) -> Result<Address, EventRegistryError> {
    let admin = storage::get_admin(env).ok_or(EventRegistryError::NotInitialized)?;
    admin.require_auth();
    Ok(admin)
}

/// Verifies the caller is the organizer of a given event.
/// Returns an error if the event doesn't exist or the caller is not the organizer.
pub fn require_organizer(
    env: &Env,
    event_id: &String,
    caller: &Address,
) -> Result<(), EventRegistryError> {
    let event_info = storage::get_event(env, event_id.clone())
        .ok_or(EventRegistryError::EventNotFound)?;
    
    if &event_info.organizer_address != caller {
        return Err(EventRegistryError::Unauthorized);
    }
    
    caller.require_auth();
    Ok(())
}
