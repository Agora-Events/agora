#![no_std]

use crate::events::{
    AgoraEvent, EventCancelledEvent, EventPostponedEvent, EventRegisteredEvent,
    EventStatusUpdatedEvent, EventsSuspendedEvent, FeeUpdatedEvent, GlobalPromoUpdatedEvent,
    InitializationEvent, InventoryIncrementedEvent, MetadataUpdatedEvent,
    OrganizerBlacklistedEvent, OrganizerRemovedFromBlacklistEvent, RegistryUpgradedEvent,
    ScannerAuthorizedEvent,
};
use crate::types::{
    BlacklistAuditEntry, EventInfo, EventRegistrationArgs, EventStatus, MultiSigConfig, PaymentInfo,
};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

pub mod error;
pub mod events;
pub mod storage;
pub mod types;

use crate::error::EventRegistryError;

#[contract]
pub struct EventRegistry;

#[contractimpl]
#[allow(deprecated)]
impl EventRegistry {
    /// Initializes the contract configuration. Can only be called once.
    /// Sets up initial admin with multi-sig configuration (threshold = 1 for single admin).
    ///
    /// # Arguments
    /// * `admin` - The administrator address.
    /// * `platform_wallet` - The platform wallet address for fees.
    /// * `platform_fee_percent` - Initial platform fee in basis points (10000 = 100%).
    pub fn initialize(
        env: Env,
        admin: Address,
        platform_wallet: Address,
        platform_fee_percent: u32,
    ) -> Result<(), EventRegistryError> {
        if storage::is_initialized(&env) {
            return Err(EventRegistryError::AlreadyInitialized);
        }

        validate_address(&env, &admin)?;
        validate_address(&env, &platform_wallet)?;

        let initial_fee = if platform_fee_percent == 0 {
            500
        } else {
            platform_fee_percent
        };

        if initial_fee > 10000 {
            return Err(EventRegistryError::InvalidFeePercent);
        }

        // Initialize multi-sig with single admin and threshold of 1
        let mut admins = Vec::new(&env);
        admins.push_back(admin.clone());
        let multisig_config = MultiSigConfig {
            admins,
            threshold: 1,
        };

        storage::set_admin(&env, &admin); // Legacy support
        storage::set_multisig_config(&env, &multisig_config);
        storage::set_platform_wallet(&env, &platform_wallet);
        storage::set_platform_fee(&env, initial_fee);
        storage::set_initialized(&env, true);

        env.events().publish(
            (AgoraEvent::ContractInitialized,),
            InitializationEvent {
                admin_address: admin,
                platform_wallet,
                platform_fee_percent: initial_fee,
                timestamp: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    /// Register a new event with organizer authentication and tiered pricing
    ///
    /// # Arguments
    /// * `event_id` - Unique identifier for the event
    /// * `organizer_address` - The wallet address of the event organizer
    /// * `payment_address` - The address where payments should be routed
    /// * `metadata_cid` - IPFS CID for event metadata
    /// * `max_supply` - Maximum number of tickets (0 = unlimited)
    /// * `tiers` - Map of tier_id to TicketTier for multi-tiered pricing
    pub fn register_event(env: Env, args: EventRegistrationArgs) -> Result<(), EventRegistryError> {
        if !storage::is_initialized(&env) {
            return Err(EventRegistryError::NotInitialized);
        }
        args.organizer_address.require_auth();

        // Check if organizer is blacklisted
        if storage::is_blacklisted(&env, &args.organizer_address) {
            return Err(EventRegistryError::OrganizerBlacklisted);
        }

        validate_metadata_cid(&env, &args.metadata_cid)?;

        if storage::event_exists(&env, args.event_id.clone()) {
            return Err(EventRegistryError::EventAlreadyExists);
        }

        // Validate tier limits don't exceed max_supply
        if args.max_supply > 0 {
            let mut total_tier_limit: i128 = 0;
            for tier in args.tiers.values() {
                total_tier_limit = total_tier_limit
                    .checked_add(tier.tier_limit)
                    .ok_or(EventRegistryError::SupplyOverflow)?;
            }
            if total_tier_limit > args.max_supply {
                return Err(EventRegistryError::TierLimitExceedsMaxSupply);
            }
        }

        // Validate resale cap if provided
        if let Some(cap) = args.resale_cap_bps {
            if cap > 10000 {
                return Err(EventRegistryError::InvalidResaleCapBps);
            }
        }

        let platform_fee_percent = storage::get_platform_fee(&env);

        let event_info = EventInfo {
            event_id: args.event_id.clone(),
            organizer_address: args.organizer_address.clone(),
            payment_address: args.payment_address.clone(),
            platform_fee_percent,
            is_active: true,
            status: EventStatus::Active,
            created_at: env.ledger().timestamp(),
            metadata_cid: args.metadata_cid.clone(),
            max_supply: args.max_supply,
            current_supply: 0,
            milestone_plan: args.milestone_plan.clone(),
            tiers: args.tiers.clone(),
            refund_deadline: args.refund_deadline,
            restocking_fee: args.restocking_fee,
            resale_cap_bps: args.resale_cap_bps,
            is_postponed: false,
            grace_period_end: 0,
        };

        storage::store_event(&env, event_info);

        env.events().publish(
            (AgoraEvent::EventRegistered,),
            EventRegisteredEvent {
                event_id: args.event_id.clone(),
                organizer_address: args.organizer_address.clone(),
                payment_address: args.payment_address.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Get event payment information including tiered pricing
    pub fn get_event_payment_info(
        env: Env,
        event_id: String,
    ) -> Result<PaymentInfo, EventRegistryError> {
        match storage::get_event(&env, event_id) {
            Some(event_info) => {
                if !event_info.is_active {
                    return Err(EventRegistryError::EventInactive);
                }
                Ok(PaymentInfo {
                    payment_address: event_info.payment_address,
                    platform_fee_percent: event_info.platform_fee_percent,
                    tiers: event_info.tiers,
                })
            }
            None => Err(EventRegistryError::EventNotFound),
        }
    }

    /// Update event status (only by organizer)
    pub fn update_event_status(
        env: Env,
        event_id: String,
        is_active: bool,
    ) -> Result<(), EventRegistryError> {
        match storage::get_event(&env, event_id.clone()) {
            Some(mut event_info) => {
                // Verify organizer signature
                event_info.organizer_address.require_auth();

                if matches!(event_info.status, EventStatus::Cancelled) {
                    return Err(EventRegistryError::EventCancelled);
                }

                // Skip storage/event writes when status is unchanged.
                if event_info.is_active == is_active {
                    return Ok(());
                }

                // Update status
                event_info.is_active = is_active;
                storage::update_event(&env, event_info.clone());

                // Emit status update event using contract event type
                env.events().publish(
                    (AgoraEvent::EventStatusUpdated,),
                    EventStatusUpdatedEvent {
                        event_id,
                        is_active,
                        updated_by: event_info.organizer_address,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                Ok(())
            }
            None => Err(EventRegistryError::EventNotFound),
        }
    }

    /// Cancel an event (only by organizer). This is irreversible.
    pub fn cancel_event(env: Env, event_id: String) -> Result<(), EventRegistryError> {
        match storage::get_event(&env, event_id.clone()) {
            Some(mut event_info) => {
                // Verify organizer signature
                event_info.organizer_address.require_auth();

                if matches!(event_info.status, EventStatus::Cancelled) {
                    return Err(EventRegistryError::EventAlreadyCancelled);
                }

                // Update status to Cancelled and deactivate
                event_info.status = EventStatus::Cancelled;
                event_info.is_active = false;
                storage::update_event(&env, event_info.clone());

                // Emit cancellation event
                env.events().publish(
                    (AgoraEvent::EventCancelled,),
                    EventCancelledEvent {
                        event_id,
                        cancelled_by: event_info.organizer_address,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                Ok(())
            }
            None => Err(EventRegistryError::EventNotFound),
        }
    }

    /// Update the decentralized metadata CID for an event (only by organizer)
    pub fn update_metadata(
        env: Env,
        event_id: String,
        new_metadata_cid: String,
    ) -> Result<(), EventRegistryError> {
        match storage::get_event(&env, event_id.clone()) {
            Some(mut event_info) => {
                // Verify organizer signature
                event_info.organizer_address.require_auth();

                // Validate new metadata CID
                validate_metadata_cid(&env, &new_metadata_cid)?;

                // Skip storage/event writes when metadata is unchanged.
                if event_info.metadata_cid == new_metadata_cid {
                    return Ok(());
                }

                // Update metadata
                event_info.metadata_cid = new_metadata_cid.clone();
                storage::update_event(&env, event_info.clone());

                // Emit metadata update event
                env.events().publish(
                    (AgoraEvent::MetadataUpdated,),
                    MetadataUpdatedEvent {
                        event_id,
                        new_metadata_cid,
                        updated_by: event_info.organizer_address,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                Ok(())
            }
            None => Err(EventRegistryError::EventNotFound),
        }
    }

    /// Stores or updates an event (legacy function for backward compatibility).
    pub fn store_event(env: Env, event_info: EventInfo) {
        // Require authorization to ensure only the organizer can store/update their event directly
        event_info.organizer_address.require_auth();
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
    pub fn set_platform_fee(env: Env, new_fee_percent: u32) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        if new_fee_percent > 10000 {
            return Err(EventRegistryError::InvalidFeePercent);
        }

        storage::set_platform_fee(&env, new_fee_percent);

        // Emit fee update event using contract event type
        env.events().publish(
            (AgoraEvent::FeeUpdated,),
            FeeUpdatedEvent { new_fee_percent },
        );

        Ok(())
    }

    /// Returns the current platform fee percentage.
    pub fn get_platform_fee(env: Env) -> u32 {
        storage::get_platform_fee(&env)
    }

    /// Returns the current administrator address.
    pub fn get_admin(env: Env) -> Result<Address, EventRegistryError> {
        storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)
    }

    /// Returns the current platform wallet address.
    pub fn get_platform_wallet(env: Env) -> Result<Address, EventRegistryError> {
        storage::get_platform_wallet(&env).ok_or(EventRegistryError::NotInitialized)
    }

    /// Sets the authorized TicketPayment contract address. Only callable by the administrator.
    ///
    /// # Arguments
    /// * `ticket_payment_address` - The address of the TicketPayment contract authorized
    ///   to call `increment_inventory`.
    pub fn set_ticket_payment_contract(
        env: Env,
        ticket_payment_address: Address,
    ) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        validate_address(&env, &ticket_payment_address)?;

        storage::set_ticket_payment_contract(&env, &ticket_payment_address);
        Ok(())
    }

    /// Returns the authorized TicketPayment contract address.
    pub fn get_ticket_payment_contract(env: Env) -> Result<Address, EventRegistryError> {
        storage::get_ticket_payment_contract(&env).ok_or(EventRegistryError::NotInitialized)
    }

    /// Increments the current_supply counter for a given event and tier.
    /// This function is restricted to calls from the authorized TicketPayment contract.
    ///
    /// # Arguments
    /// * `event_id` - The event whose inventory to increment.
    /// * `tier_id` - The tier whose inventory to increment.
    ///
    /// # Errors
    /// * `UnauthorizedCaller` - If the invoker is not the registered TicketPayment contract.
    /// * `EventNotFound` - If no event with the given ID exists.
    /// * `EventInactive` - If the event is not currently active.
    /// * `TierNotFound` - If the tier does not exist.
    /// * `TierSupplyExceeded` - If the tier's limit has been reached.
    /// * `MaxSupplyExceeded` - If the event's max supply has been reached (when max_supply > 0).
    /// * `SupplyOverflow` - If incrementing would cause an i128 overflow.
    pub fn increment_inventory(
        env: Env,
        event_id: String,
        tier_id: String,
        quantity: u32,
    ) -> Result<(), EventRegistryError> {
        let ticket_payment_addr =
            storage::get_ticket_payment_contract(&env).ok_or(EventRegistryError::NotInitialized)?;
        ticket_payment_addr.require_auth();

        if quantity == 0 {
            return Err(EventRegistryError::InvalidQuantity);
        }

        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        if !event_info.is_active || matches!(event_info.status, EventStatus::Cancelled) {
            return Err(EventRegistryError::EventInactive);
        }

        let quantity_i128 = quantity as i128;

        // Check global supply limits
        if event_info.max_supply > 0 {
            let new_total_supply = event_info
                .current_supply
                .checked_add(quantity_i128)
                .ok_or(EventRegistryError::SupplyOverflow)?;
            if new_total_supply > event_info.max_supply {
                return Err(EventRegistryError::MaxSupplyExceeded);
            }
        }

        // Get and update tier
        let mut tier = event_info
            .tiers
            .get(tier_id.clone())
            .ok_or(EventRegistryError::TierNotFound)?;

        let new_tier_sold = tier
            .current_sold
            .checked_add(quantity_i128)
            .ok_or(EventRegistryError::SupplyOverflow)?;

        if new_tier_sold > tier.tier_limit {
            return Err(EventRegistryError::TierSupplyExceeded);
        }

        tier.current_sold = new_tier_sold;
        event_info.tiers.set(tier_id, tier);

        event_info.current_supply = event_info
            .current_supply
            .checked_add(quantity_i128)
            .ok_or(EventRegistryError::SupplyOverflow)?;

        let new_supply = event_info.current_supply;
        storage::update_event(&env, event_info);

        env.events().publish(
            (AgoraEvent::InventoryIncremented,),
            InventoryIncrementedEvent {
                event_id,
                new_supply,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Decrements the current_supply counter for a given event and tier.
    /// This function is restricted to calls from the authorized TicketPayment contract upon refund.
    ///
    /// # Arguments
    /// * `event_id` - The event whose inventory to decrement.
    /// * `tier_id` - The tier whose inventory to decrement.
    ///
    /// # Errors
    /// * `UnauthorizedCaller` - If the invoker is not the registered TicketPayment contract.
    /// * `EventNotFound` - If no event with the given ID exists.
    /// * `TierNotFound` - If the tier does not exist.
    /// * `SupplyUnderflow` - If decrementing would cause the supply to go below 0.
    pub fn decrement_inventory(
        env: Env,
        event_id: String,
        tier_id: String,
    ) -> Result<(), EventRegistryError> {
        let ticket_payment_addr =
            storage::get_ticket_payment_contract(&env).ok_or(EventRegistryError::NotInitialized)?;
        ticket_payment_addr.require_auth();

        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        // Get and update tier
        let mut tier = event_info
            .tiers
            .get(tier_id.clone())
            .ok_or(EventRegistryError::TierNotFound)?;

        if tier.current_sold <= 0 {
            return Err(EventRegistryError::SupplyUnderflow);
        }

        tier.current_sold = tier
            .current_sold
            .checked_sub(1)
            .ok_or(EventRegistryError::SupplyUnderflow)?;

        event_info.tiers.set(tier_id, tier);

        if event_info.current_supply <= 0 {
            return Err(EventRegistryError::SupplyUnderflow);
        }

        event_info.current_supply = event_info
            .current_supply
            .checked_sub(1)
            .ok_or(EventRegistryError::SupplyUnderflow)?;

        let new_supply = event_info.current_supply;
        storage::update_event(&env, event_info);

        env.events().publish(
            (crate::events::AgoraEvent::InventoryDecremented,),
            crate::events::InventoryDecrementedEvent {
                event_id,
                new_supply,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Upgrades the contract to a new WASM hash. Only callable by the administrator.
    /// Performs post-upgrade state verification to ensure critical storage is intact.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        // Post-upgrade state verification
        let verified_admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        storage::get_platform_wallet(&env).ok_or(EventRegistryError::NotInitialized)?;

        env.events().publish(
            (AgoraEvent::ContractUpgraded,),
            RegistryUpgradedEvent {
                admin_address: verified_admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Adds an organizer to the blacklist with mandatory audit logging.
    /// Only callable by the administrator.
    pub fn blacklist_organizer(
        env: Env,
        organizer_address: Address,
        reason: String,
    ) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        validate_address(&env, &organizer_address)?;

        // Check if already blacklisted
        if storage::is_blacklisted(&env, &organizer_address) {
            return Err(EventRegistryError::OrganizerBlacklisted);
        }

        // Add to blacklist
        storage::add_to_blacklist(&env, &organizer_address);

        // Create audit log entry
        let audit_entry = BlacklistAuditEntry {
            organizer_address: organizer_address.clone(),
            added_to_blacklist: true,
            admin_address: admin.clone(),
            reason: reason.clone(),
            timestamp: env.ledger().timestamp(),
        };
        storage::add_blacklist_audit_entry(&env, audit_entry);

        // Emit blacklist event
        env.events().publish(
            (AgoraEvent::OrganizerBlacklisted,),
            OrganizerBlacklistedEvent {
                organizer_address: organizer_address.clone(),
                admin_address: admin.clone(),
                reason: reason.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );

        // Suspend all active events from this organizer
        suspend_organizer_events(env.clone(), organizer_address)?;

        Ok(())
    }

    /// Removes an organizer from the blacklist with mandatory audit logging.
    /// Only callable by the administrator.
    pub fn remove_from_blacklist(
        env: Env,
        organizer_address: Address,
        reason: String,
    ) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        validate_address(&env, &organizer_address)?;

        // Check if currently blacklisted
        if !storage::is_blacklisted(&env, &organizer_address) {
            return Err(EventRegistryError::OrganizerNotBlacklisted);
        }

        // Remove from blacklist
        storage::remove_from_blacklist(&env, &organizer_address);

        // Create audit log entry
        let audit_entry = BlacklistAuditEntry {
            organizer_address: organizer_address.clone(),
            added_to_blacklist: false,
            admin_address: admin.clone(),
            reason: reason.clone(),
            timestamp: env.ledger().timestamp(),
        };
        storage::add_blacklist_audit_entry(&env, audit_entry);

        // Emit removal event
        env.events().publish(
            (AgoraEvent::OrganizerRemovedFromBlacklist,),
            OrganizerRemovedFromBlacklistEvent {
                organizer_address,
                admin_address: admin,
                reason,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Checks if an organizer is blacklisted.
    pub fn is_organizer_blacklisted(env: Env, organizer_address: Address) -> bool {
        storage::is_blacklisted(&env, &organizer_address)
    }

    /// Retrieves the blacklist audit log.
    pub fn get_blacklist_audit_log(env: Env) -> Vec<BlacklistAuditEntry> {
        storage::get_blacklist_audit_log(&env)
    }

    /// Sets a platform-wide promotional discount. Only callable by the administrator.
    /// The promo automatically expires when the ledger timestamp passes `promo_expiry`.
    ///
    /// # Arguments
    /// * `global_promo_bps` - Discount rate in basis points (e.g., 1500 = 15% off). 0 clears the promo.
    /// * `promo_expiry` - Unix timestamp after which the promo is no longer applied.
    pub fn set_global_promo(
        env: Env,
        global_promo_bps: u32,
        promo_expiry: u64,
    ) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        if global_promo_bps > 10000 {
            return Err(EventRegistryError::InvalidPromoBps);
        }

        storage::set_global_promo_bps(&env, global_promo_bps);
        storage::set_promo_expiry(&env, promo_expiry);

        env.events().publish(
            (AgoraEvent::GlobalPromoUpdated,),
            GlobalPromoUpdatedEvent {
                global_promo_bps,
                promo_expiry,
                admin_address: admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Returns the current global promotional discount rate in basis points.
    pub fn get_global_promo_bps(env: Env) -> u32 {
        storage::get_global_promo_bps(&env)
    }

    /// Returns the expiry timestamp for the current global promo.
    pub fn get_promo_expiry(env: Env) -> u64 {
        storage::get_promo_expiry(&env)
    }

    /// Marks an event as postponed and sets a temporary refund grace period.
    /// During this window, all guests may request refunds regardless of their
    /// ticket tier's standard refundability rules or refund deadlines.
    pub fn postpone_event(
        env: Env,
        event_id: String,
        grace_period_end: u64,
    ) -> Result<(), EventRegistryError> {
        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        // Only the organizer may postpone their event.
        event_info.organizer_address.require_auth();

        let now = env.ledger().timestamp();
        if grace_period_end <= now {
            return Err(EventRegistryError::InvalidGracePeriodEnd);
        }

        event_info.is_postponed = true;
        event_info.grace_period_end = grace_period_end;
        storage::update_event(&env, event_info.clone());

        env.events().publish(
            (AgoraEvent::EventPostponed,),
            EventPostponedEvent {
                event_id,
                organizer_address: event_info.organizer_address,
                grace_period_end,
                timestamp: now,
            },
        );

        Ok(())
    }

    /// Authorizes a new scanner wallet for a specific event
    pub fn authorize_scanner(
        env: Env,
        event_id: String,
        scanner: Address,
    ) -> Result<(), EventRegistryError> {
        let event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        // Only the organizer can authorize scanners
        event_info.organizer_address.require_auth();

        storage::authorize_scanner(&env, event_id.clone(), &scanner);

        env.events().publish(
            (AgoraEvent::ScannerAuthorized,),
            ScannerAuthorizedEvent {
                event_id,
                scanner,
                authorized_by: event_info.organizer_address,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Checks if a scanner is authorized for a specific event
    pub fn is_scanner_authorized(env: Env, event_id: String, scanner: Address) -> bool {
        storage::is_scanner_authorized(&env, event_id, &scanner)
    }
}

fn validate_address(env: &Env, address: &Address) -> Result<(), EventRegistryError> {
    if address == &env.current_contract_address() {
        return Err(EventRegistryError::InvalidAddress);
    }
    Ok(())
}

fn validate_metadata_cid(env: &Env, cid: &String) -> Result<(), EventRegistryError> {
    if cid.len() < 46 {
        return Err(EventRegistryError::InvalidMetadataCid);
    }

    // We expect CIDv1 base32, which starts with 'b'
    // Convert to Bytes to check the first character safely
    let mut bytes = soroban_sdk::Bytes::new(env);
    bytes.append(&cid.clone().into());

    if !bytes.is_empty() && bytes.get(0) != Some(b'b') {
        return Err(EventRegistryError::InvalidMetadataCid);
    }

    Ok(())
}

/// Suspends all active events for a blacklisted organizer.
/// This implements the "Suspension" ripple effect.
fn suspend_organizer_events(
    env: Env,
    organizer_address: Address,
) -> Result<(), EventRegistryError> {
    let organizer_events = storage::get_organizer_events(&env, &organizer_address);
    let mut suspended_count = 0u32;

    for event_id in organizer_events.iter() {
        if let Some(mut event_info) = storage::get_event(&env, event_id.clone()) {
            if event_info.is_active {
                event_info.is_active = false;
                storage::store_event(&env, event_info);
                suspended_count += 1;
            }
        }
    }

    // Emit suspension event if any events were suspended
    if suspended_count > 0 {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::EventsSuspended,),
            EventsSuspendedEvent {
                organizer_address,
                suspended_event_count: suspended_count,
                admin_address: admin,
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    Ok(())
}

#[cfg(test)]
mod test;

// TODO: Uncomment when multisig functions are implemented
// #[cfg(test)]
// mod test_multisig;
