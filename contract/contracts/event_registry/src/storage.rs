//! # Storage Module
//!
//! This module is the single point of contact between the EventRegistry contract
//! logic and the Soroban persistent ledger. Every read and write to on-chain state
//! goes through a thin wrapper function defined here, keeping the rest of the codebase
//! free of raw storage calls.
//!
//! ## Storage type
//!
//! **All** state in this contract uses env.storage().persistent(). Persistent
//! storage survives ledger expiry (unlike temporary storage, which is automatically
//! deleted after a TTL) and is therefore appropriate for long-lived contract data such
//! as event records, organizer indexes, and governance proposals.
//!
//! There is **no** use of env.storage().instance() or env.storage().temporary()
//! in this module.
//!
//! ## Storage keys
//!
//! All keys are variants of the DataKey enum (crate::types::DataKey), annotated
//! with #[contracttype] so Soroban can serialise them as XDR values.
//!
//! | Key variant | Value type | Purpose |
//! |---|---|---|
//! | Admin | Address | Legacy single-admin address |
//! | MultiSigConfig | MultiSigConfig | Multi-admin governance config |
//! | PlatformWallet | Address | Fee-collection wallet |
//! | PlatformFee | u32 | Global platform fee in basis points |
//! | Initialized | bool | One-time initialisation guard |
//! | Event(event_id) | EventInfo | Full event record keyed by event ID |
//! | OrganizerEvent(addr, event_id) | bool | Membership flag for organizer index |
//! | OrganizerEventShard(addr, shard) | Vec<String> | Sharded event ID list per organizer |
//! | OrganizerEventCount(addr) | u32 | Total events registered by an organizer |
//! | TicketPaymentContract | Address | Authorised TicketPayment contract |
//! | BlacklistedOrganizer(addr) | bool | Blacklist membership flag |
//! | BlacklistLog | Vec<BlacklistAuditEntry> | Append-only audit log |
//! | GlobalPromoBps | u32 | Platform-wide discount rate in basis points |
//! | PromoExpiry | u64 | Unix timestamp when the promo expires |
//! | EventReceipt(event_id) | EventReceipt | Minimal receipt for an archived event |
//! | OrganizerReceipt(addr, event_id) | bool | Membership flag for receipt index |
//! | OrganizerReceiptShard(addr, shard) | Vec<String> | Sharded receipt ID list per organizer |
//! | OrganizerReceiptCount(addr) | u32 | Total archived receipts for an organizer |
//! | ProposalCounter | u64 | Auto-incrementing governance proposal ID |
//! | Proposal(id) | Proposal | Governance proposal keyed by ID |
//! | ActiveProposals | Vec<u64> | IDs of proposals not yet executed |
//! | AuthorizedScanner(event_id, addr) | bool | Scanner authorisation flag per event |
//! | Series(series_id) | SeriesRegistry | Series grouping of events |
//! | SeriesPass(pass_id) | SeriesPass | Season pass for a series |
//! | HolderSeriesPass(addr, series_id) | String | Maps (holder, series) to pass_id |
//! | SeriesEvent(series_id, event_id) | bool | Fast membership check for series events |
//! | GuestProfile(addr) | GuestProfile | Per-attendee loyalty profile |
//! | OrganizerStake(addr) | OrganizerStake | Organizer collateral stake record |
//! | MinStakeAmount | i128 | Minimum stake required for Verified status |
//! | StakingToken | Address | Token contract accepted for staking |
//! | TotalStaked | i128 | Sum of all currently staked tokens |
//! | StakersList | Vec<Address> | All addresses with an active stake |
//! | TokenWhitelist(addr) | bool | Payment token whitelist membership flag |
//! | GlobalEventCount | u32 | Cumulative count of all registered events |
//! | GlobalTicketsSold | i128 | Cumulative count of all tickets sold |
//!
//! ## Sharding strategy
//!
//! Per-organizer event and receipt lists are split into fixed-size buckets of
//! SHARD_SIZE entries each. This prevents any single storage entry from
//! growing unboundedly and exceeding Soroban per-entry size limits. The shard
//! index for a new item is count / SHARD_SIZE, where count is the current
//! total for that organizer.
use crate::types::{
    BlacklistAuditEntry, DataKey, EventInfo, GuestProfile, MultiSigConfig, OrganizerStake, Proposal,
};
use crate::types::{SeriesPass, SeriesRegistry};
use soroban_sdk::{vec, Address, Env, String, Vec};
// ── Series Storage ────────────────────────────────────────────────────────────
/// Persists a SeriesRegistry and indexes every event it contains.
/// Storage keys: DataKey::Series(series_id) and DataKey::SeriesEvent(series_id, event_id).
/// Storage type: Persistent
pub fn store_series(env: &Env, series: &SeriesRegistry) {
    env.storage()
        .persistent()
        .set(&DataKey::Series(series.series_id.clone()), series);
    // Index each event_id for fast lookup
    for event_id in series.event_ids.iter() {
        env.storage().persistent().set(
            &DataKey::SeriesEvent(series.series_id.clone(), event_id.clone()),
            &true,
        );
    }
}

/// Returns the SeriesRegistry for the given series_id, or None.
/// Storage key: DataKey::Series(series_id). Storage type: Persistent
pub fn get_series(env: &Env, series_id: String) -> Option<SeriesRegistry> {
    env.storage().persistent().get(&DataKey::Series(series_id))
}

/// Returns true if event_id is a member of series_id. O(1) lookup via DataKey::SeriesEvent.
/// Storage type: Persistent
pub fn series_contains_event(env: &Env, series_id: String, event_id: String) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::SeriesEvent(series_id, event_id))
}

// ── Series Pass Storage ────────────────────────────────────────────────────────
/// Persists a SeriesPass and writes a reverse-lookup index (holder, series_id) -> pass_id.
/// Storage keys: DataKey::SeriesPass(pass_id) and DataKey::HolderSeriesPass(holder, series_id).
/// Storage type: Persistent
pub fn store_series_pass(env: &Env, pass: &SeriesPass) {
    env.storage()
        .persistent()
        .set(&DataKey::SeriesPass(pass.pass_id.clone()), pass);
    env.storage().persistent().set(
        &DataKey::HolderSeriesPass(pass.holder.clone(), pass.series_id.clone()),
        &pass.pass_id,
    );
}

/// Returns the SeriesPass with the given pass_id, or None.
/// Storage key: DataKey::SeriesPass(pass_id). Storage type: Persistent
pub fn get_series_pass(env: &Env, pass_id: String) -> Option<SeriesPass> {
    env.storage()
        .persistent()
        .get(&DataKey::SeriesPass(pass_id))
}

pub fn get_holder_series_pass(
    env: &Env,
    holder: &Address,
    series_id: String,
) -> Option<SeriesPass> {
    if let Some(pass_id) = env
        .storage()
        .persistent()
        .get::<_, String>(&DataKey::HolderSeriesPass(
            holder.clone(),
            series_id.clone(),
        ))
    {
        env.storage()
            .persistent()
            .get(&DataKey::SeriesPass(pass_id))
    } else {
        None
    }
}

/// Increments usage count for a pass, enforcing usage limit. Returns Some(pass) if incremented, None if not allowed.
pub fn increment_series_pass_usage(env: &Env, pass_id: String) -> Option<SeriesPass> {
    if let Some(mut pass) = get_series_pass(env, pass_id.clone()) {
        if pass.usage_count < pass.usage_limit {
            pass.usage_count += 1;
            store_series_pass(env, &pass);
            Some(pass)
        } else {
            None // Usage limit reached
        }
    } else {
        None
    }
}

const SHARD_SIZE: u32 = 50;

fn sync_active_event_count(env: &Env, existing: Option<&EventInfo>, updated: &EventInfo) {
    // Private events are excluded from the global active event counter.
    if updated.is_private {
        // If the event was previously public and is now private, undo any active count.
        if let Some(prev) = existing {
            if !prev.is_private && prev.is_active {
                decrement_global_active_event_count(env);
            }
        }
        return;
    }

    match existing {
        Some(previous) if previous.is_active && !updated.is_active => {
            decrement_global_active_event_count(env);
        }
        Some(previous) if !previous.is_active && updated.is_active => {
            increment_global_active_event_count(env);
        }
        // Transitioning from private to public: treat as a fresh active event if active.
        Some(previous) if previous.is_private && updated.is_active => {
            increment_global_active_event_count(env);
        }
        None if updated.is_active => {
            increment_global_active_event_count(env);
        }
        _ => {}
    }
}

/// Sets the administrator address of the contract (legacy function).
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

/// Retrieves the administrator address of the contract (legacy function).
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

/// Sets the multi-signature configuration.
pub fn set_multisig_config(env: &Env, config: &MultiSigConfig) {
    env.storage()
        .persistent()
        .set(&DataKey::MultiSigConfig, config);
}

/// Retrieves the multi-signature configuration.
pub fn get_multisig_config(env: &Env) -> Option<MultiSigConfig> {
    env.storage().persistent().get(&DataKey::MultiSigConfig)
}

/// Checks if an address is an admin.
pub fn is_admin(env: &Env, address: &Address) -> bool {
    if let Some(config) = get_multisig_config(env) {
        for admin in config.admins.iter() {
            if admin == *address {
                return true;
            }
        }
    }
    false
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

/// Gets the next proposal ID and increments the counter.
pub fn get_next_proposal_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::ProposalCounter)
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::ProposalCounter, &(current + 1));
    current
}

/// Gets the current proposal counter without incrementing.
pub fn get_proposal_counter(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::ProposalCounter)
        .unwrap_or(0)
}

/// Sets the proposal counter.
pub fn set_proposal_counter(env: &Env, counter: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::ProposalCounter, &counter);
}

/// Stores a proposal.
pub fn set_proposal(env: &Env, proposal: &Proposal) {
    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal.proposal_id), proposal);
}

/// Stores a proposal (legacy name for compatibility).
pub fn store_proposal(env: &Env, proposal: &Proposal) {
    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal.proposal_id), proposal);

    // Add to active proposals list if not executed
    if !proposal.executed {
        let mut active_proposals: Vec<u64> = get_active_proposals(env);
        let mut exists = false;
        for id in active_proposals.iter() {
            if id == proposal.proposal_id {
                exists = true;
                break;
            }
        }
        if !exists {
            active_proposals.push_back(proposal.proposal_id);
            env.storage()
                .persistent()
                .set(&DataKey::ActiveProposals, &active_proposals);
        }
    }
}

/// Retrieves a proposal by ID.
pub fn get_proposal(env: &Env, proposal_id: u64) -> Option<Proposal> {
    env.storage()
        .persistent()
        .get(&DataKey::Proposal(proposal_id))
}

/// Retrieves all active proposal IDs.
pub fn get_active_proposals(env: &Env) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ActiveProposals)
        .unwrap_or_else(|| Vec::new(env))
}

/// Adds a proposal to the active proposals list.
pub fn add_active_proposal(env: &Env, proposal_id: u64) {
    let mut active_proposals = get_active_proposals(env);
    // Check if already exists
    for id in active_proposals.iter() {
        if id == proposal_id {
            return;
        }
    }
    active_proposals.push_back(proposal_id);
    env.storage()
        .persistent()
        .set(&DataKey::ActiveProposals, &active_proposals);
}

/// Removes a proposal from the active list (when executed or expired).
pub fn remove_active_proposal(env: &Env, proposal_id: u64) {
    let active_proposals: Vec<u64> = get_active_proposals(env);
    let mut new_proposals = Vec::new(env);

    for id in active_proposals.iter() {
        if id != proposal_id {
            new_proposals.push_back(id);
        }
    }

    env.storage()
        .persistent()
        .set(&DataKey::ActiveProposals, &new_proposals);
}

/// Removes a proposal from the active list (legacy name for compatibility).
pub fn remove_from_active_proposals(env: &Env, proposal_id: u64) {
    let active_proposals: Vec<u64> = get_active_proposals(env);
    let mut new_proposals = Vec::new(env);

    for id in active_proposals.iter() {
        if id != proposal_id {
            new_proposals.push_back(id);
        }
    }

    env.storage()
        .persistent()
        .set(&DataKey::ActiveProposals, &new_proposals);
}

/// Stores a new event or updates an existing one.
/// Also updates the organizer's list of events.
pub fn store_event(env: &Env, event_info: EventInfo) {
    let event_id = event_info.event_id.clone();
    let organizer = event_info.organizer_address.clone();
    let existing = get_event(env, event_id.clone());

    sync_active_event_count(env, existing.as_ref(), &event_info);

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

        // Increment global event counter only for public events.
        if !event_info.is_private {
            increment_global_event_count(env);
        }
    }
}

/// Updates event data without touching organizer index.
/// Use this for mutations on already-registered events.
pub fn update_event(env: &Env, event_info: EventInfo) {
    let event_id = event_info.event_id.clone();
    let existing = get_event(env, event_id.clone());

    sync_active_event_count(env, existing.as_ref(), &event_info);

    env.storage()
        .persistent()
        .set(&DataKey::Event(event_id), &event_info);
}

/// Integrates storage functions to get, remove events and handle their receipts.
pub fn get_event(env: &Env, event_id: String) -> Option<EventInfo> {
    env.storage().persistent().get(&DataKey::Event(event_id))
}

/// Removes an event and cleans up organizer indexes
pub fn remove_event(env: &Env, event_id: String) {
    if let Some(event_info) = get_event(env, event_id.clone()) {
        let organizer = event_info.organizer_address;

        if event_info.is_active && !event_info.is_private {
            decrement_global_active_event_count(env);
        }

        // Remove from main mapping
        env.storage()
            .persistent()
            .remove(&DataKey::Event(event_id.clone()));

        // Remove from organizer's individual event record
        env.storage().persistent().remove(&DataKey::OrganizerEvent(
            organizer.clone(),
            event_id.clone(),
        ));

        // Remove from organizer's sharded list
        remove_from_organizer_index(env, &organizer, event_id);
    }
}

/// Helper to remove an event_id from an organizer's sharded index
fn remove_from_organizer_index(env: &Env, organizer: &Address, event_id: String) {
    let count = get_organizer_event_count(env, organizer);
    if count == 0 {
        return;
    }

    let num_shards = count.div_ceil(SHARD_SIZE);
    let mut found = false;

    for i in 0..num_shards {
        let shard: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::OrganizerEventShard(organizer.clone(), i))
            .unwrap_or_else(|| vec![env]);

        let mut found_in_shard = false;
        let mut new_shard = vec![env];

        for id in shard.iter() {
            if id == event_id {
                found_in_shard = true;
                found = true;
            } else {
                new_shard.push_back(id);
            }
        }

        if found_in_shard {
            env.storage().persistent().set(
                &DataKey::OrganizerEventShard(organizer.clone(), i),
                &new_shard,
            );
            break;
        }
    }

    if found {
        env.storage().persistent().set(
            &DataKey::OrganizerEventCount(organizer.clone()),
            &(count - 1),
        );
    }
}

/// Stores an event receipt
pub fn store_event_receipt(env: &Env, receipt: crate::types::EventReceipt) {
    let organizer = receipt.organizer_address.clone();
    let event_id = receipt.event_id.clone();

    env.storage()
        .persistent()
        .set(&DataKey::EventReceipt(event_id.clone()), &receipt);

    if !has_organizer_receipt(env, &organizer, event_id.clone()) {
        let count = get_organizer_receipt_count(env, &organizer);
        let shard_id = count / SHARD_SIZE;

        let mut shard: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::OrganizerReceiptShard(organizer.clone(), shard_id))
            .unwrap_or_else(|| vec![env]);

        shard.push_back(event_id.clone());
        env.storage().persistent().set(
            &DataKey::OrganizerReceiptShard(organizer.clone(), shard_id),
            &shard,
        );

        env.storage().persistent().set(
            &DataKey::OrganizerReceiptCount(organizer.clone()),
            &(count + 1),
        );

        env.storage()
            .persistent()
            .set(&DataKey::OrganizerReceipt(organizer, event_id), &true);
    }
}

/// Retrieves an event receipt by its event_id
pub fn get_event_receipt(env: &Env, event_id: String) -> Option<crate::types::EventReceipt> {
    env.storage()
        .persistent()
        .get(&DataKey::EventReceipt(event_id))
}

/// Retrieves all archived receipts associated with an organizer.
pub fn get_organizer_receipts(env: &Env, organizer: &Address) -> Vec<crate::types::EventReceipt> {
    let count = get_organizer_receipt_count(env, organizer);
    let mut receipts = vec![env];

    if count == 0 {
        return receipts;
    }

    let num_shards = count.div_ceil(SHARD_SIZE);
    for i in 0..num_shards {
        let shard: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::OrganizerReceiptShard(organizer.clone(), i))
            .unwrap_or_else(|| vec![env]);

        for event_id in shard.iter() {
            if let Some(receipt) = get_event_receipt(env, event_id) {
                receipts.push_back(receipt);
            }
        }
    }

    receipts
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

/// Retrieves the total number of archived event receipts for an organizer.
pub fn get_organizer_receipt_count(env: &Env, organizer: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::OrganizerReceiptCount(organizer.clone()))
        .unwrap_or(0)
}

/// Checks if an organizer has a specific archived receipt in their index.
pub fn has_organizer_receipt(env: &Env, organizer: &Address, event_id: String) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::OrganizerReceipt(organizer.clone(), event_id))
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

/// Authorizes a scanner for an event.
pub fn authorize_scanner(env: &Env, event_id: String, scanner: &Address) {
    env.storage().persistent().set(
        &DataKey::AuthorizedScanner(event_id, scanner.clone()),
        &true,
    );
}

/// Removes authorization for a scanner from an event.
pub fn remove_scanner(env: &Env, event_id: String, scanner: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::AuthorizedScanner(event_id, scanner.clone()));
}

/// Checks if a scanner is authorized for an event.
pub fn is_scanner_authorized(env: &Env, event_id: String, scanner: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::AuthorizedScanner(event_id, scanner.clone()))
        .unwrap_or(false)
}

// ── Loyalty & Staking Storage ─────────────────────────────────────────────────

/// Retrieves a guest's loyalty profile.
pub fn get_guest_profile(env: &Env, guest: &Address) -> Option<GuestProfile> {
    env.storage()
        .persistent()
        .get(&DataKey::GuestProfile(guest.clone()))
}

/// Stores (creates or updates) a guest's loyalty profile.
pub fn set_guest_profile(env: &Env, profile: &GuestProfile) {
    env.storage().persistent().set(
        &DataKey::GuestProfile(profile.guest_address.clone()),
        profile,
    );
}

/// Retrieves an organizer's stake record.
pub fn get_organizer_stake(env: &Env, organizer: &Address) -> Option<OrganizerStake> {
    env.storage()
        .persistent()
        .get(&DataKey::OrganizerStake(organizer.clone()))
}

/// Stores (creates or updates) an organizer's stake record.
pub fn set_organizer_stake(env: &Env, stake: &OrganizerStake) {
    env.storage()
        .persistent()
        .set(&DataKey::OrganizerStake(stake.organizer.clone()), stake);
}

/// Sets the contract administrator address for organizer whitelisting.
pub fn set_contract_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::ContractAdmin, admin);
}

/// Retrieves the contract administrator address for organizer whitelisting.
pub fn get_contract_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::ContractAdmin)
}

/// Approves or removes an organizer from the whitelist.
pub fn set_approved_organizer(env: &Env, organizer: &Address, approved: bool) {
    if approved {
        env.storage()
            .instance()
            .set(&DataKey::ApprovedOrganizer(organizer.clone()), &true);
    } else {
        env.storage()
            .instance()
            .remove(&DataKey::ApprovedOrganizer(organizer.clone()));
    }
}

/// Checks if an organizer is approved.
pub fn is_approved_organizer(env: &Env, addr: &Address) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::ApprovedOrganizer(addr.clone()))
        .unwrap_or(false)
}

/// Removes an organizer's stake record (used on unstake).
pub fn remove_organizer_stake(env: &Env, organizer: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::OrganizerStake(organizer.clone()));
}

/// Gets the minimum stake amount required for Verified status.
pub fn get_min_stake_amount(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::MinStakeAmount)
        .unwrap_or(0)
}

/// Sets the minimum stake amount required for Verified status.
pub fn set_min_stake_amount(env: &Env, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::MinStakeAmount, &amount);
}

/// Gets the token contract address accepted for staking.
pub fn get_staking_token(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::StakingToken)
}

/// Sets the token contract address accepted for staking.
pub fn set_staking_token(env: &Env, token: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::StakingToken, token);
}

/// Gets the total amount currently staked across all organizers.
pub fn get_total_staked(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalStaked)
        .unwrap_or(0)
}

/// Adds `amount` to the total staked counter.
pub fn add_to_total_staked(env: &Env, amount: i128) {
    let current = get_total_staked(env);
    env.storage()
        .persistent()
        .set(&DataKey::TotalStaked, &(current + amount));
}

/// Subtracts `amount` from the total staked counter.
pub fn subtract_from_total_staked(env: &Env, amount: i128) {
    let current = get_total_staked(env);
    let new_val = current.saturating_sub(amount);
    env.storage()
        .persistent()
        .set(&DataKey::TotalStaked, &new_val);
}

/// Gets the list of all currently staked organizer addresses.
pub fn get_stakers_list(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::StakersList)
        .unwrap_or_else(|| Vec::new(env))
}

/// Adds an organizer to the stakers list if not already present.
pub fn add_to_stakers_list(env: &Env, organizer: &Address) {
    let mut list = get_stakers_list(env);
    for addr in list.iter() {
        if addr == *organizer {
            return; // already in list
        }
    }
    list.push_back(organizer.clone());
    env.storage().persistent().set(&DataKey::StakersList, &list);
}

/// Removes an organizer from the stakers list.
pub fn remove_from_stakers_list(env: &Env, organizer: &Address) {
    let list = get_stakers_list(env);
    let mut new_list = Vec::new(env);
    for addr in list.iter() {
        if addr != *organizer {
            new_list.push_back(addr);
        }
    }
    env.storage()
        .persistent()
        .set(&DataKey::StakersList, &new_list);
}

// ── Token Whitelist Storage ───────────────────────────────────────────────────

/// Adds a token address to the payment token whitelist.
pub fn add_to_token_whitelist(env: &Env, token: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::TokenWhitelist(token.clone()), &true);
}

/// Removes a token address from the payment token whitelist.
pub fn remove_from_token_whitelist(env: &Env, token: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::TokenWhitelist(token.clone()));
}

/// Returns true if the given token address is whitelisted for payments.
pub fn is_token_whitelisted(env: &Env, token: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::TokenWhitelist(token.clone()))
        .unwrap_or(false)
}

// ── Event-Specific Token Whitelist ─────────────────────────────────────────────

/// Adds a token address to the event-specific payment token whitelist.
pub fn add_event_token_whitelist(env: &Env, event_id: String, token: &Address) {
    env.storage().persistent().set(
        &DataKey::EventTokenWhitelist(event_id, token.clone()),
        &true,
    );
}

/// Removes a token address from the event-specific payment token whitelist.
pub fn remove_event_token_whitelist(env: &Env, event_id: String, token: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::EventTokenWhitelist(event_id, token.clone()));
}

/// Returns true if the given token address is whitelisted for payments to a specific event.
pub fn is_event_token_whitelisted(env: &Env, event_id: String, token: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::EventTokenWhitelist(event_id, token.clone()))
        .unwrap_or(false)
}

/// Returns true if the given token address is accepted for payments to the event,
/// checking either event-specific whitelist or global whitelist based on event configuration.
pub fn is_token_accepted_for_event(
    env: &Env,
    _event_id: String,
    token: &Address,
    use_global_whitelist: bool,
    accepted_tokens: &Vec<Address>,
) -> bool {
    if use_global_whitelist {
        // Use global whitelist
        is_token_whitelisted(env, token)
    } else if accepted_tokens.is_empty() {
        // No specific tokens configured, fall back to global whitelist
        is_token_whitelisted(env, token)
    } else {
        // Check if token is in event-specific whitelist
        accepted_tokens.contains(token)
    }
}

// ── Global Counters ──────────────────────────────────────────────────────────

/// Returns the total number of events ever registered on the platform.
pub fn get_global_event_count(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::GlobalEventCount)
        .unwrap_or(0)
}

/// Increments the global event counter by one.
pub fn increment_global_event_count(env: &Env) {
    let current = get_global_event_count(env);
    env.storage()
        .persistent()
        .set(&DataKey::GlobalEventCount, &(current + 1));
}

/// Returns the total number of currently active events on the platform.
pub fn get_global_active_event_count(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::GlobalActiveEventCount)
        .unwrap_or(0)
}

/// Increments the global active event counter by one.
pub fn increment_global_active_event_count(env: &Env) {
    let current = get_global_active_event_count(env);
    env.storage()
        .persistent()
        .set(&DataKey::GlobalActiveEventCount, &(current + 1));
}

/// Decrements the global active event counter by one.
pub fn decrement_global_active_event_count(env: &Env) {
    let current = get_global_active_event_count(env);
    env.storage()
        .persistent()
        .set(&DataKey::GlobalActiveEventCount, &current.saturating_sub(1));
}

/// Returns the total number of tickets sold across all events.
pub fn get_global_tickets_sold(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::GlobalTicketsSold)
        .unwrap_or(0)
}

/// Adds `quantity` to the global tickets sold counter.
pub fn add_to_global_tickets_sold(env: &Env, quantity: i128) {
    let current = get_global_tickets_sold(env);
    env.storage().persistent().set(
        &DataKey::GlobalTicketsSold,
        &(current.saturating_add(quantity)),
    );
}

/// Subtracts `quantity` from the global tickets sold counter.
pub fn subtract_from_global_tickets_sold(env: &Env, quantity: i128) {
    let current = get_global_tickets_sold(env);
    env.storage().persistent().set(
        &DataKey::GlobalTicketsSold,
        &(current.saturating_sub(quantity)),
    );
}

/// Returns the number of tickets a user has purchased for a specific event tier.
pub fn get_user_ticket_count(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    user: &Address,
) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::UserTicketCount(
            event_id.clone(),
            tier_id.clone(),
            user.clone(),
        ))
        .unwrap_or(0)
}

/// Sets the number of tickets a user has purchased for a specific event tier.
pub fn set_user_ticket_count(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    user: &Address,
    count: u32,
) {
    env.storage().persistent().set(
        &DataKey::UserTicketCount(event_id.clone(), tier_id.clone(), user.clone()),
        &count,
    );
}

/// Adds `quantity` to the user's ticket count for a specific event tier.
pub fn add_to_user_ticket_count(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    user: &Address,
    quantity: u32,
) {
    let current = get_user_ticket_count(env, event_id, tier_id, user);
    set_user_ticket_count(
        env,
        event_id,
        tier_id,
        user,
        current.saturating_add(quantity),
    );
}

/// Subtracts `quantity` from the user's ticket count for a specific event tier.
pub fn subtract_from_user_ticket_count(
    env: &Env,
    event_id: &String,
    tier_id: &String,
    user: &Address,
    quantity: u32,
) {
    let current = get_user_ticket_count(env, event_id, tier_id, user);
    set_user_ticket_count(
        env,
        event_id,
        tier_id,
        user,
        current.saturating_sub(quantity),
    );
}

// ── Waitlist Storage ──────────────────────────────────────────────────────────

/// Check if a user is already on the waitlist for an event.
/// Storage key: DataKey::Waitlist(event_id, user). Storage type: Persistent
pub fn is_on_waitlist(env: &Env, event_id: &String, user: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Waitlist(event_id.clone(), user.clone()))
        .unwrap_or(false)
}

/// Add a user to the waitlist for an event.
/// Storage key: DataKey::Waitlist(event_id, user) -> true. Storage type: Persistent
pub fn add_to_waitlist(env: &Env, event_id: &String, user: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::Waitlist(event_id.clone(), user.clone()), &true);
}

// ── Event Pause Storage ────────────────────────────────────────────────────────

/// Returns whether an event is paused. Returns false if the pause status is not set (i.e., event is not paused).
/// Storage key: DataKey::EventPaused(event_id). Storage type: Persistent
pub fn is_event_paused(env: &Env, event_id: &String) -> bool {
    env.storage()
        .persistent()
        .get::<_, bool>(&DataKey::EventPaused(event_id.clone()))
        .unwrap_or(false)
}

/// Sets the pause status for an event.
/// Storage key: DataKey::EventPaused(event_id). Storage type: Persistent
pub fn set_event_paused(env: &Env, event_id: &String, is_paused: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::EventPaused(event_id.clone()), &is_paused);
}
