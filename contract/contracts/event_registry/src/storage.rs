use crate::types::{BlacklistAuditEntry, DataKey, EventInfo};
use soroban_sdk::{vec, Address, Env, String, Vec};

const SHARD_SIZE: u32 = 50;

/// Sets the administrator address of the contract.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

/// Retrieves the administrator address of the contract.
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

/// Sets the platform wallet address of the contract.
pub fn set_platform_wallet(env: &Env, wallet: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::PlatformWallet, wallet);
}

/// Retrieves the platform wallet address of the contract.
pub fn get_platform_wallet(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::PlatformWallet)
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

/// Sets initialization flag.
pub fn set_initialized(env: &Env, value: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::Initialized, &value);
}

/// Checks if contract has been initialized.
pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Initialized)
        .unwrap_or(false)
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

    // Update organizer's event index if it doesn't exist
    if !has_organizer_event(env, &organizer, event_id.clone()) {
        let count = get_organizer_event_count(env, &organizer);
        let shard_id = count / SHARD_SIZE;

        let mut shard: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::OrganizerEventShard(organizer.clone(), shard_id))
            .unwrap_or_else(|| vec![env]);

        shard.push_back(event_id.clone());
        env.storage().persistent().set(
            &DataKey::OrganizerEventShard(organizer.clone(), shard_id),
            &shard,
        );

        env.storage().persistent().set(
            &DataKey::OrganizerEventCount(organizer.clone()),
            &(count + 1),
        );

        env.storage()
            .persistent()
            .set(&DataKey::OrganizerEvent(organizer, event_id), &true);
    }
}

/// Updates event data without touching organizer index.
/// Use this for mutations on already-registered events.
pub fn update_event(env: &Env, event_info: EventInfo) {
    let event_id = event_info.event_id.clone();
    env.storage()
        .persistent()
        .set(&DataKey::Event(event_id), &event_info);
}

/// Retrieves event information by event_id.
pub fn get_event(env: &Env, event_id: String) -> Option<EventInfo> {
    env.storage().persistent().get(&DataKey::Event(event_id))
}

/// Checks if an event with the given event_id exists.
pub fn event_exists(env: &Env, event_id: String) -> bool {
    env.storage().persistent().has(&DataKey::Event(event_id))
}

/// Retrieves the total number of events for an organizer.
pub fn get_organizer_event_count(env: &Env, organizer: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::OrganizerEventCount(organizer.clone()))
        .unwrap_or(0)
}

/// Checks if an organizer has a specific event in their index.
pub fn has_organizer_event(env: &Env, organizer: &Address, event_id: String) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::OrganizerEvent(organizer.clone(), event_id))
}

/// Retrieves all event_ids associated with an organizer by iterating through shards.
/// NOTE: For very large lists, this may exceed gas limits. Use shard-based iteration for scale.
pub fn get_organizer_events(env: &Env, organizer: &Address) -> Vec<String> {
    let count = get_organizer_event_count(env, organizer);
    let mut all_events = vec![env];

    if count == 0 {
        return all_events;
    }

    let num_shards = count.div_ceil(SHARD_SIZE);
    for i in 0..num_shards {
        let shard: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::OrganizerEventShard(organizer.clone(), i))
            .unwrap_or_else(|| vec![env]);
        for id in shard.iter() {
            all_events.push_back(id);
        }
    }
    all_events
}

/// Retrieves a specific shard of event_ids for an organizer.
pub fn get_organizer_event_shard(env: &Env, organizer: &Address, shard_id: u32) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&DataKey::OrganizerEventShard(organizer.clone(), shard_id))
        .unwrap_or_else(|| vec![env])
}

/// Sets the authorized TicketPayment contract address.
pub fn set_ticket_payment_contract(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::TicketPaymentContract, address);
}

/// Retrieves the authorized TicketPayment contract address.
pub fn get_ticket_payment_contract(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::TicketPaymentContract)
}

/// Checks if an organizer is blacklisted.
pub fn is_blacklisted(env: &Env, organizer: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::BlacklistedOrganizer(organizer.clone()))
        .unwrap_or(false)
}

/// Adds an organizer to the blacklist.
pub fn add_to_blacklist(env: &Env, organizer: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::BlacklistedOrganizer(organizer.clone()), &true);
}

/// Removes an organizer from the blacklist.
pub fn remove_from_blacklist(env: &Env, organizer: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::BlacklistedOrganizer(organizer.clone()));
}

/// Adds an audit log entry for blacklist actions.
pub fn add_blacklist_audit_entry(env: &Env, entry: BlacklistAuditEntry) {
    let mut audit_log: Vec<BlacklistAuditEntry> = get_blacklist_audit_log(env);
    audit_log.push_back(entry);
    env.storage()
        .persistent()
        .set(&DataKey::BlacklistLog, &audit_log);
}

/// Retrieves the blacklist audit log.
pub fn get_blacklist_audit_log(env: &Env) -> Vec<BlacklistAuditEntry> {
    env.storage()
        .persistent()
        .get(&DataKey::BlacklistLog)
        .unwrap_or_else(|| Vec::new(env))
}

/// Sets the global promotional discount in basis points.
pub fn set_global_promo_bps(env: &Env, bps: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::GlobalPromoBps, &bps);
}

/// Retrieves the global promotional discount in basis points.
pub fn get_global_promo_bps(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::GlobalPromoBps)
        .unwrap_or(0)
}

/// Sets the expiry timestamp for the global promotional discount.
pub fn set_promo_expiry(env: &Env, expiry: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::PromoExpiry, &expiry);
}

/// Retrieves the expiry timestamp for the global promotional discount.
pub fn get_promo_expiry(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::PromoExpiry)
        .unwrap_or(0)
}
