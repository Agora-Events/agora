#![no_std]

use crate::events::{
    AgoraEvent, CollateralStakedEvent, CollateralUnstakedEvent, CustomFeeSetEvent,
    EventArchivedEvent, EventCancelledEvent, EventPostponedEvent, EventRegisteredEvent,
    EventStatusUpdatedEvent, EventsSuspendedEvent, FeeUpdatedEvent, GlobalPromoUpdatedEvent,
    GoalMetEvent, InitializationEvent, InventoryIncrementedEvent, LoyaltyScoreUpdatedEvent,
    MetadataUpdatedEvent, OrganizerBlacklistedEvent, OrganizerRemovedFromBlacklistEvent,
    RegistryUpgradedEvent, ScannerAuthorizedEvent, StakerRewardsClaimedEvent,
    StakerRewardsDistributedEvent,
};
use crate::types::{
    BlacklistAuditEntry, EventInfo, EventReceipt, EventRegistrationArgs, EventStatus, GuestProfile,
    MultiSigConfig, OrganizerStake, PaymentInfo,
};
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, String, Vec};

pub mod error;
pub mod events;
pub mod storage;
pub mod types;

use crate::types::{SeriesPass, SeriesRegistry};

use crate::error::EventRegistryError;

#[contract]
pub struct EventRegistry;

#[contractimpl]
#[allow(deprecated)]
impl EventRegistry {
    /// Register a new series grouping multiple events
    pub fn register_series(
        env: Env,
        series_id: String,
        name: String,
        event_ids: Vec<String>,
        organizer_address: Address,
        metadata_cid: Option<String>,
    ) -> Result<(), EventRegistryError> {
        organizer_address.require_auth();
        // Validate all event_ids exist and belong to organizer
        for event_id in event_ids.iter() {
            let event = storage::get_event(&env, event_id.clone())
                .ok_or(EventRegistryError::EventNotFound)?;
            if event.organizer_address != organizer_address {
                return Err(EventRegistryError::UnauthorizedCaller);
            }
        }
        let series = SeriesRegistry {
            series_id: series_id.clone(),
            name,
            event_ids: event_ids.clone(),
            organizer_address: organizer_address.clone(),
            metadata_cid,
        };
        storage::store_series(&env, &series);
        Ok(())
    }

    /// Get a series by ID
    pub fn get_series(env: Env, series_id: String) -> Option<SeriesRegistry> {
        storage::get_series(&env, series_id)
    }

    /// Issue a season pass for a series
    pub fn issue_series_pass(
        env: Env,
        pass_id: String,
        series_id: String,
        holder: Address,
        usage_limit: u32,
        expires_at: u64,
    ) -> Result<(), EventRegistryError> {
        // Only organizer of the series can issue passes
        let series = storage::get_series(&env, series_id.clone())
            .ok_or(EventRegistryError::EventNotFound)?;
        series.organizer_address.require_auth();
        let pass = SeriesPass {
            pass_id: pass_id.clone(),
            series_id: series_id.clone(),
            holder: holder.clone(),
            usage_limit,
            usage_count: 0,
            issued_at: env.ledger().timestamp(),
            expires_at,
        };
        storage::store_series_pass(&env, &pass);
        Ok(())
    }

    /// Get a pass by ID
    pub fn get_series_pass(env: Env, pass_id: String) -> Option<SeriesPass> {
        storage::get_series_pass(&env, pass_id)
    }

    /// Get a pass for a holder and series
    pub fn get_holder_series_pass(
        env: Env,
        holder: Address,
        series_id: String,
    ) -> Option<SeriesPass> {
        storage::get_holder_series_pass(&env, &holder, series_id)
    }
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
            min_sales_target: args.min_sales_target.unwrap_or(0),
            target_deadline: args.target_deadline.unwrap_or(0),
            goal_met: false,
            custom_fee_bps: None,
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
                    custom_fee_bps: event_info.custom_fee_bps,
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

    /// Archive an event that is settled and no longer active.
    /// Wipes large data structures and leaves a minimal Receipt,
    /// returning reclaimed XLM deposit to the organizer automatically.
    pub fn archive_event(env: Env, event_id: String) -> Result<(), EventRegistryError> {
        match storage::get_event(&env, event_id.clone()) {
            Some(event_info) => {
                event_info.organizer_address.require_auth();

                if event_info.is_active {
                    return Err(EventRegistryError::EventIsActive);
                }

                storage::remove_event(&env, event_id.clone());

                let receipt = EventReceipt {
                    event_id: event_id.clone(),
                    organizer_address: event_info.organizer_address.clone(),
                    total_sold: event_info.current_supply,
                    archived_at: env.ledger().timestamp(),
                };
                storage::store_event_receipt(&env, receipt);

                env.events().publish(
                    (AgoraEvent::EventArchived,),
                    EventArchivedEvent {
                        event_id,
                        organizer_address: event_info.organizer_address,
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

    /// Sets a custom fee for a specific event. Only callable by the administrator.
    pub fn set_custom_event_fee(
        env: Env,
        event_id: String,
        custom_fee_bps: Option<u32>,
    ) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        if let Some(fee) = custom_fee_bps {
            if fee > 10000 {
                return Err(EventRegistryError::InvalidFeePercent);
            }
        }

        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        event_info.custom_fee_bps = custom_fee_bps;
        storage::update_event(&env, event_info);

        // Emit custom fee set event
        env.events().publish(
            (AgoraEvent::CustomFeeSet,),
            CustomFeeSetEvent {
                event_id,
                custom_fee_bps,
                admin_address: admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
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

        // Check if goal met now
        if !event_info.goal_met
            && event_info.min_sales_target > 0
            && event_info.current_supply >= event_info.min_sales_target
        {
            event_info.goal_met = true;
            env.events().publish(
                (AgoraEvent::GoalMet,),
                GoalMetEvent {
                    event_id: event_id.clone(),
                    min_sales_target: event_info.min_sales_target,
                    current_supply: event_info.current_supply,
                    timestamp: env.ledger().timestamp(),
                },
            );
        }

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

    // ── Loyalty & Staking ──────────────────────────────────────────────────────

    /// Configures staking parameters. Only callable by the admin.
    ///
    /// # Arguments
    /// * `token` - Token contract address accepted for staking
    /// * `min_amount` - Minimum token amount to stake to achieve Verified status
    pub fn set_staking_config(
        env: Env,
        token: Address,
        min_amount: i128,
    ) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        if min_amount <= 0 {
            return Err(EventRegistryError::InvalidStakeAmount);
        }

        storage::set_staking_token(&env, &token);
        storage::set_min_stake_amount(&env, min_amount);
        Ok(())
    }

    /// Allows an organizer to stake collateral tokens to unlock Verified status.
    /// The organizer must approve this contract to spend `amount` of the staking token
    /// before calling this function.
    ///
    /// # Arguments
    /// * `organizer` - The organizer's wallet address (must sign)
    /// * `amount` - Amount of staking token to lock
    pub fn stake_collateral(
        env: Env,
        organizer: Address,
        amount: i128,
    ) -> Result<(), EventRegistryError> {
        organizer.require_auth();

        if amount <= 0 {
            return Err(EventRegistryError::InvalidStakeAmount);
        }

        if storage::get_organizer_stake(&env, &organizer).is_some() {
            return Err(EventRegistryError::AlreadyStaked);
        }

        let token =
            storage::get_staking_token(&env).ok_or(EventRegistryError::StakingNotConfigured)?;
        let min_amount = storage::get_min_stake_amount(&env);

        // Transfer tokens from organizer to this contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &organizer,
            &env.current_contract_address(),
            &amount,
        );

        let is_verified = amount >= min_amount;

        let stake = OrganizerStake {
            organizer: organizer.clone(),
            token: token.clone(),
            amount,
            staked_at: env.ledger().timestamp(),
            is_verified,
            reward_balance: 0,
            total_rewards_claimed: 0,
        };

        storage::set_organizer_stake(&env, &stake);
        storage::add_to_total_staked(&env, amount);
        storage::add_to_stakers_list(&env, &organizer);

        env.events().publish(
            (AgoraEvent::CollateralStaked,),
            CollateralStakedEvent {
                organizer,
                token,
                amount,
                is_verified,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Allows an organizer to unstake their collateral and reclaim their tokens.
    /// All accrued rewards must be claimed before unstaking.
    ///
    /// # Arguments
    /// * `organizer` - The organizer's wallet address (must sign)
    pub fn unstake_collateral(env: Env, organizer: Address) -> Result<(), EventRegistryError> {
        organizer.require_auth();

        let stake =
            storage::get_organizer_stake(&env, &organizer).ok_or(EventRegistryError::NotStaked)?;

        // Transfer tokens back to organizer
        let token_client = token::Client::new(&env, &stake.token);
        token_client.transfer(&env.current_contract_address(), &organizer, &stake.amount);

        storage::subtract_from_total_staked(&env, stake.amount);
        storage::remove_organizer_stake(&env, &organizer);
        storage::remove_from_stakers_list(&env, &organizer);

        env.events().publish(
            (AgoraEvent::CollateralUnstaked,),
            CollateralUnstakedEvent {
                organizer,
                token: stake.token,
                amount: stake.amount,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Distributes rewards proportionally to all active stakers based on their
    /// share of the total staked amount. The caller must approve the reward tokens
    /// to this contract before calling.
    ///
    /// This should be called by the admin periodically based on ticket sales volume,
    /// or by an authorized contract (e.g., TicketPayment) after settling fees.
    ///
    /// # Arguments
    /// * `caller` - Admin or authorized contract address
    /// * `token` - Token to distribute as rewards (must match staking token)
    /// * `total_reward` - Total reward amount to distribute across all stakers
    pub fn distribute_staker_rewards(
        env: Env,
        caller: Address,
        total_reward: i128,
    ) -> Result<(), EventRegistryError> {
        caller.require_auth();

        // Only admin can call this function
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        if caller != admin {
            return Err(EventRegistryError::Unauthorized);
        }

        if total_reward <= 0 {
            return Err(EventRegistryError::InvalidRewardAmount);
        }

        let token =
            storage::get_staking_token(&env).ok_or(EventRegistryError::StakingNotConfigured)?;

        let total_staked = storage::get_total_staked(&env);
        if total_staked == 0 {
            return Err(EventRegistryError::NotStaked);
        }

        // Transfer reward tokens from caller to this contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &caller,
            &env.current_contract_address(),
            &total_reward,
        );

        // Distribute proportionally to each staker
        let stakers = storage::get_stakers_list(&env);
        let staker_count = stakers.len();

        for organizer in stakers.iter() {
            if let Some(mut stake) = storage::get_organizer_stake(&env, &organizer) {
                // reward = total_reward * stake.amount / total_staked
                let reward = total_reward
                    .checked_mul(stake.amount)
                    .and_then(|v| v.checked_div(total_staked))
                    .unwrap_or(0);
                if reward > 0 {
                    stake.reward_balance = stake.reward_balance.saturating_add(reward);
                    storage::set_organizer_stake(&env, &stake);
                }
            }
        }

        env.events().publish(
            (AgoraEvent::StakerRewardsDistributed,),
            StakerRewardsDistributedEvent {
                total_reward,
                staker_count,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Allows an organizer to claim their accumulated staking rewards.
    ///
    /// # Arguments
    /// * `organizer` - The organizer's wallet address (must sign)
    pub fn claim_staker_rewards(env: Env, organizer: Address) -> Result<i128, EventRegistryError> {
        organizer.require_auth();

        let mut stake =
            storage::get_organizer_stake(&env, &organizer).ok_or(EventRegistryError::NotStaked)?;

        if stake.reward_balance == 0 {
            return Err(EventRegistryError::NoRewardsAvailable);
        }

        let reward_to_claim = stake.reward_balance;

        // Transfer reward tokens to organizer
        let token_client = token::Client::new(&env, &stake.token);
        token_client.transfer(
            &env.current_contract_address(),
            &organizer,
            &reward_to_claim,
        );

        stake.total_rewards_claimed = stake.total_rewards_claimed.saturating_add(reward_to_claim);
        stake.reward_balance = 0;
        storage::set_organizer_stake(&env, &stake);

        env.events().publish(
            (AgoraEvent::StakerRewardsClaimed,),
            StakerRewardsClaimedEvent {
                organizer,
                amount: reward_to_claim,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(reward_to_claim)
    }

    /// Returns the stake record for an organizer, or None if not staked.
    pub fn get_organizer_stake(env: Env, organizer: Address) -> Option<OrganizerStake> {
        storage::get_organizer_stake(&env, &organizer)
    }

    /// Returns true if the organizer has staked the minimum required amount.
    pub fn is_organizer_verified(env: Env, organizer: Address) -> bool {
        storage::get_organizer_stake(&env, &organizer)
            .map(|s| s.is_verified)
            .unwrap_or(false)
    }

    /// Updates the loyalty score for a guest after a ticket purchase.
    /// Callable by the admin or the authorized TicketPayment contract.
    ///
    /// # Arguments
    /// * `caller` - Admin or authorized TicketPayment contract address
    /// * `guest` - Guest wallet address
    /// * `tickets_purchased` - Number of tickets purchased in this transaction
    /// * `amount_spent` - Amount spent in this transaction (in token stroops)
    pub fn update_loyalty_score(
        env: Env,
        caller: Address,
        guest: Address,
        tickets_purchased: u32,
        amount_spent: i128,
    ) -> Result<(), EventRegistryError> {
        caller.require_auth();

        // Only admin or authorized ticket payment contract can update loyalty scores
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        let ticket_payment_contract = storage::get_ticket_payment_contract(&env);

        let is_authorized = caller == admin
            || ticket_payment_contract
                .as_ref()
                .map(|c| c == &caller)
                .unwrap_or(false);

        if !is_authorized {
            return Err(EventRegistryError::Unauthorized);
        }

        if tickets_purchased == 0 {
            return Err(EventRegistryError::InvalidQuantity);
        }

        let mut profile = storage::get_guest_profile(&env, &guest).unwrap_or(GuestProfile {
            guest_address: guest.clone(),
            loyalty_score: 0,
            total_tickets_purchased: 0,
            total_spent: 0,
            last_updated: 0,
        });

        // Award 10 points per ticket purchased
        let points_earned = (tickets_purchased as u64).saturating_mul(10);
        profile.loyalty_score = profile.loyalty_score.saturating_add(points_earned);
        profile.total_tickets_purchased = profile
            .total_tickets_purchased
            .saturating_add(tickets_purchased);
        profile.total_spent = profile.total_spent.saturating_add(amount_spent);
        profile.last_updated = env.ledger().timestamp();

        storage::set_guest_profile(&env, &profile);

        env.events().publish(
            (AgoraEvent::LoyaltyScoreUpdated,),
            LoyaltyScoreUpdatedEvent {
                guest,
                new_score: profile.loyalty_score,
                tickets_purchased,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Returns the guest's loyalty profile, or None if no profile exists.
    pub fn get_guest_profile(env: Env, guest: Address) -> Option<GuestProfile> {
        storage::get_guest_profile(&env, &guest)
    }

    /// Returns the platform-fee discount in basis points for a guest based on
    /// their current loyalty score.
    ///
    /// Score tiers:
    /// - Score  0  –  99 : 0 bps  (no discount)
    /// - Score 100 – 499 : 250 bps (2.5% off platform fee)
    /// - Score 500 – 999 : 500 bps (5% off platform fee)
    /// - Score 1000+     : 1000 bps (10% off platform fee)
    pub fn get_loyalty_discount_bps(env: Env, guest: Address) -> u32 {
        let score = storage::get_guest_profile(&env, &guest)
            .map(|p| p.loyalty_score)
            .unwrap_or(0);

        if score >= 1000 {
            1000
        } else if score >= 500 {
            500
        } else if score >= 100 {
            250
        } else {
            0
        }
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

#[cfg(test)]
mod test_e2e;

// TODO: Uncomment when multisig functions are implemented
// #[cfg(test)]
// mod test_multisig;
