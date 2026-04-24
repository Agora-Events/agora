//! # Event Registry Contract
//!
//! ## Overview
//!
//! The `event-registry` crate is the central on-chain registry for the Agora Events platform,
//! deployed on the [Soroban](https://soroban.stellar.org) smart-contract runtime on Stellar.
//! It is the single source of truth for every event that exists on the platform: creation,
//! status transitions, inventory tracking, organizer management, and fee configuration all
//! live here.
//!
//! ## Architecture
//!
//! The crate is organised into five focused modules:
//!
//! | Module | Responsibility |
//! |--------|---------------|
//! | [`lib`](crate) | Public contract entry-points exposed via `#[contractimpl]` |
//! | [`types`] | All `#[contracttype]` structs and enums shared across modules |
//! | [`storage`] | Thin wrappers around `env.storage()` – one function per data key |
//! | [`events`] | Soroban event structs and the [`AgoraEvent`](crate::events::AgoraEvent) topic enum |
//! | [`error`] | The [`EventRegistryError`](crate::error::EventRegistryError) enum returned by every fallible function |
//!
//! ### Key data types
//!
//! * [`EventInfo`](crate::types::EventInfo) – the full on-chain record for a registered event,
//!   including tiered pricing ([`TicketTier`](crate::types::TicketTier)), supply counters,
//!   milestone plans, and status flags.
//! * [`MultiSigConfig`](crate::types::MultiSigConfig) – multi-admin governance configuration
//!   with a configurable approval threshold.
//! * [`OrganizerStake`](crate::types::OrganizerStake) – collateral staked by organizers to
//!   unlock *Verified* status and earn proportional staking rewards.
//! * [`GuestProfile`](crate::types::GuestProfile) – per-attendee loyalty score and spend history.
//! * [`SeriesRegistry`](crate::types::SeriesRegistry) / [`SeriesPass`](crate::types::SeriesPass) –
//!   grouping of related events into a series with reusable season passes.
//!
//! ### Storage strategy
//!
//! All state is kept in **persistent** storage so that it survives ledger expiry.  Large
//! per-organizer lists (event IDs, receipts) are sharded into fixed-size buckets of 50 entries
//! each to stay within Soroban's per-entry size limits.
//!
//! ## Usage
//!
//! ### Initialisation
//!
//! The contract must be initialised exactly once by calling [`EventRegistry::initialize`]:
//!
//! ```text
//! EventRegistry::initialize(env, admin, platform_wallet, platform_fee_bps, usdc_token)
//! ```
//!
//! This sets the admin, platform wallet, default fee rate (in basis points), and automatically
//! whitelists the provided USDC token for payments.
//!
//! ### Registering an event
//!
//! Organizers call [`EventRegistry::register_event`] with an
//! [`EventRegistrationArgs`](crate::types::EventRegistrationArgs) struct that bundles the event
//! ID, payment address, IPFS metadata CID, supply cap, and one or more
//! [`TicketTier`](crate::types::TicketTier) entries.
//!
//! ### Interaction with TicketPayment
//!
//! `EventRegistry` and the companion `TicketPayment` contract work together to process ticket
//! sales while keeping inventory consistent:
//!
//! 1. **Registration** – the platform admin calls
//!    [`EventRegistry::set_ticket_payment_contract`] once to record the `TicketPayment`
//!    contract address on-chain.
//! 2. **Purchase flow** – when a buyer purchases a ticket, `TicketPayment` handles the token
//!    transfer and fee split, then calls [`EventRegistry::increment_inventory`] to atomically
//!    increment the per-tier and global supply counters.  `EventRegistry` enforces that only
//!    the registered `TicketPayment` address may call this function
//!    (`ticket_payment_addr.require_auth()`), preventing unauthorised supply manipulation.
//! 3. **Refund flow** – when a refund is approved, `TicketPayment` calls
//!    [`EventRegistry::decrement_inventory`] to roll back the supply counters, again gated
//!    behind the same address check.
//! 4. **Payment info** – `TicketPayment` can query [`EventRegistry::get_event_payment_info`]
//!    to retrieve the current fee rates and tier pricing for a given event before processing
//!    a payment.
//!
//! This separation of concerns means `EventRegistry` never touches tokens directly; all
//! financial logic lives in `TicketPayment`, while `EventRegistry` remains the authoritative
//! registry for event state and inventory.

#![no_std]

use crate::events::{
    AdminUpdatedEvent, AgoraEvent, CollateralStakedEvent, CollateralUnstakedEvent,
    CustomFeeSetEvent, EventArchivedEvent, EventCancelledEvent, EventPostponedEvent,
    EventRegisteredEvent, EventStatusUpdatedEvent, EventsSuspendedEvent, FeeUpdatedEvent,
    FeedbackCidSetEvent, GlobalPromoUpdatedEvent, GoalMetEvent, InitializationEvent,
    InventoryIncrementedEvent, LoyaltyScoreUpdatedEvent, MetadataUpdatedEvent,
    OrganizerBlacklistedEvent, OrganizerRemovedFromBlacklistEvent, RegistryUpgradedEvent,
    ScannerAuthorizedEvent, StakerRewardsClaimedEvent, StakerRewardsDistributedEvent,
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

const MIN_METADATA_CID_LEN: u32 = 46;
const MAX_METADATA_CID_LEN: u32 = 100;
pub const VERSION: u32 = 1;

#[contract]
pub struct EventRegistry;

#[contractimpl]
#[allow(deprecated)]
impl EventRegistry {
    /// Returns the current version of the contract for off-chain services.
    pub fn get_version(_env: Env) -> u32 {
        VERSION
    }

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
            if matches!(event.status, EventStatus::Cancelled) {
                return Err(EventRegistryError::EventCancelled);
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
    /// The `usdc_token` address is automatically added to the payment token whitelist.
    ///
    /// # Arguments
    /// * `admin` - The administrator address.
    /// * `platform_wallet` - The platform wallet address for fees.
    /// * `platform_fee_percent` - Initial platform fee in basis points (10000 = 100%).
    /// * `usdc_token` - The USDC token contract address, automatically whitelisted on init.
    pub fn initialize(
        env: Env,
        admin: Address,
        platform_wallet: Address,
        platform_fee_percent: u32,
        usdc_token: Address,
    ) -> Result<(), EventRegistryError> {
        if storage::is_initialized(&env) {
            return Err(EventRegistryError::AlreadyInitialized);
        }

        validate_address(&env, &admin)?;
        validate_address(&env, &platform_wallet)?;
        validate_address(&env, &usdc_token)?;

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
        // Automatically whitelist the USDC token provided at initialization
        storage::add_to_token_whitelist(&env, &usdc_token);
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

    /// Adds a token address to the payment token whitelist. Only callable by the administrator.
    pub fn add_to_token_whitelist(env: Env, token: Address) -> Result<(), EventRegistryError> {
        require_admin(&env)?;
        validate_address(&env, &token)?;
        storage::add_to_token_whitelist(&env, &token);
        Ok(())
    }

    /// Removes a token address from the payment token whitelist. Only callable by the administrator.
    pub fn remove_from_token_whitelist(env: Env, token: Address) -> Result<(), EventRegistryError> {
        require_admin(&env)?;
        storage::remove_from_token_whitelist(&env, &token);
        Ok(())
    }

    /// Returns true if the given token address is whitelisted for payments.
    pub fn is_token_whitelisted(env: Env, token: Address) -> bool {
        storage::is_token_whitelisted(&env, &token)
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
        validate_address(&env, &args.organizer_address)?;
        args.organizer_address.require_auth();

        // Check if organizer is blacklisted
        if storage::is_blacklisted(&env, &args.organizer_address) {
            return Err(EventRegistryError::OrganizerBlacklisted);
        }

        validate_metadata_cid(&env, &args.metadata_cid)?;

        if storage::event_exists(&env, args.event_id.clone()) {
            return Err(EventRegistryError::EventAlreadyExists);
        }

        // Validate tier limits and supply consistency
        let mut total_tier_limit: i128 = 0;
        for tier in args.tiers.values() {
            if tier.tier_limit < 0 {
                return Err(EventRegistryError::InvalidQuantity);
            }
            total_tier_limit = total_tier_limit
                .checked_add(tier.tier_limit)
                .ok_or(EventRegistryError::SupplyOverflow)?;

            // Combined validation: Ensure restocking fee does not exceed any tier price
            if args.restocking_fee > 0 && args.restocking_fee > tier.price {
                return Err(EventRegistryError::RestockingFeeExceedsTicketPrice);
            }
        }

        if args.max_supply > 0 && total_tier_limit > args.max_supply {
            return Err(EventRegistryError::TierLimitExceedsMaxSupply);
        }

        // Validate resale cap if provided
        if let Some(cap) = args.resale_cap_bps {
            if cap > 10000 {
                return Err(EventRegistryError::InvalidResaleCapBps);
            }
        }

        validate_restocking_fee(&args)?;

        // Validate milestone plan: total release_percent must not exceed 10000 bps (100%)
        if let Some(ref milestones) = args.milestone_plan {
            let mut total_release: u32 = 0;
            for m in milestones.iter() {
                total_release = total_release
                    .checked_add(m.release_percent)
                    .ok_or(EventRegistryError::SupplyOverflow)?;
            }
            if total_release > 10000 {
                return Err(EventRegistryError::InvalidMilestonePlan);
            }
        }

        // Validate tags: max 10 tags, each at most 32 characters
        if let Some(ref tags) = args.tags {
            if tags.len() > 10 {
                return Err(EventRegistryError::InvalidTags);
            }
            for tag in tags.iter() {
                if tag.len() > 32 {
                    return Err(EventRegistryError::InvalidTags);
                }
            }
        }

        if let Some(deadline) = args.target_deadline {
            if deadline <= env.ledger().timestamp() {
                return Err(EventRegistryError::InvalidTargetDeadline);
            }
        }

        let platform_fee_percent = storage::get_platform_fee(&env);

        // Sanitize: trim leading/trailing whitespace from the event name
        let trimmed_name = trim_string(&env, &args.name);

        let event_info = EventInfo {
            event_id: args.event_id.clone(),
            name: trimmed_name,
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
            banner_cid: args.banner_cid,
            tags: args.tags,
            start_time: args.start_time,
            is_private: args.is_private,
            end_time: args.end_time,
            feedback_cid: None,
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

                if matches!(event_info.status, EventStatus::Cancelled) {
                    return Err(EventRegistryError::EventCancelled);
                }

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

    /// Sets the post-event feedback CID for an event.
    ///
    /// Can only be called by the event organizer after the event's `end_time` has passed.
    /// If `end_time` is 0 (not set), the call is rejected.
    ///
    /// # Arguments
    /// * `event_id` - The event to attach feedback to.
    /// * `feedback_cid` - IPFS CID pointing to the post-event feedback content.
    ///
    /// # Errors
    /// * `EventNotFound` - If no event with the given ID exists.
    /// * `EventCancelled` - If the event has been cancelled.
    /// * `EventNotEnded` - If `end_time` is 0 or the current ledger time is before `end_time`.
    /// * `InvalidMetadataCid` - If the provided CID fails format validation.
    pub fn set_feedback_cid(
        env: Env,
        event_id: String,
        feedback_cid: String,
    ) -> Result<(), EventRegistryError> {
        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        event_info.organizer_address.require_auth();

        if matches!(event_info.status, EventStatus::Cancelled) {
            return Err(EventRegistryError::EventCancelled);
        }

        let now = env.ledger().timestamp();
        if event_info.end_time == 0 || now < event_info.end_time {
            return Err(EventRegistryError::EventNotEnded);
        }

        validate_metadata_cid(&env, &feedback_cid)?;

        event_info.feedback_cid = Some(feedback_cid.clone());
        storage::update_event(&env, event_info.clone());

        env.events().publish(
            (AgoraEvent::FeedbackCidSet,),
            FeedbackCidSetEvent {
                event_id,
                feedback_cid,
                updated_by: event_info.organizer_address,
                timestamp: now,
            },
        );

        Ok(())
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

    /// Returns the organizer address for a given event ID, or `None` if the event does not exist.
    pub fn get_organizer_address(env: Env, event_id: String) -> Option<Address> {
        storage::get_event(&env, event_id).map(|e| e.organizer_address)
    }

    /// Returns the total number of tickets sold for an event across all tiers.
    pub fn get_total_tickets_sold(env: Env, event_id: String) -> Result<i128, EventRegistryError> {
        let event = storage::get_event(&env, event_id).ok_or(EventRegistryError::EventNotFound)?;
        Ok(event.current_supply)
    }

    /// Returns the total number of events that have ever been registered on the platform.
    pub fn get_managed_events_count(env: Env) -> u32 {
        storage::get_global_event_count(&env)
    }

    /// Returns the total number of events that are currently active.
    pub fn get_active_events_count(env: Env) -> u32 {
        storage::get_global_active_event_count(&env)
    }

    /// Returns the total number of tickets sold across all events on the platform.
    pub fn get_global_tickets_sold(env: Env) -> i128 {
        storage::get_global_tickets_sold(&env)
    }

    /// Checks if an event exists.
    pub fn event_exists(env: Env, event_id: String) -> bool {
        storage::event_exists(&env, event_id)
    }

    /// Retrieves all event IDs for an organizer.
    pub fn get_organizer_events(env: Env, organizer: Address) -> Vec<String> {
        storage::get_organizer_events(&env, &organizer)
    }

    /// Retrieves all archived event receipts for an organizer.
    pub fn get_organizer_receipts(env: Env, organizer: Address) -> Vec<EventReceipt> {
        storage::get_organizer_receipts(&env, &organizer)
    }

    /// Updates the platform fee percentage. Only callable by the administrator.
    pub fn set_platform_fee(env: Env, new_fee_percent: u32) -> Result<(), EventRegistryError> {
        require_admin(&env)?;

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

    /// Returns the current minimum stake amount required for Verified organizer status.
    pub fn get_min_stake_amount(env: Env) -> i128 {
        storage::get_min_stake_amount(&env)
    }

    /// Sets a custom fee for a specific event. Only callable by the administrator.
    pub fn set_custom_event_fee(
        env: Env,
        event_id: String,
        custom_fee_bps: Option<u32>,
    ) -> Result<(), EventRegistryError> {
        let admin = require_admin(&env)?;

        if let Some(fee) = custom_fee_bps {
            if fee > 10000 {
                return Err(EventRegistryError::InvalidFeePercent);
            }
        }

        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        if matches!(event_info.status, EventStatus::Cancelled) {
            return Err(EventRegistryError::EventCancelled);
        }

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

    /// Updates the administrator address. Only callable by the current administrator.
    /// Emits an `AdminUpdated` event with the old and new admin addresses.
    ///
    /// # Arguments
    /// * `new_admin` - The new administrator address.
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), EventRegistryError> {
        let old_admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        old_admin.require_auth();

        validate_address(&env, &new_admin)?;

        storage::set_admin(&env, &new_admin);

        env.events().publish(
            (AgoraEvent::AdminUpdated,),
            AdminUpdatedEvent {
                old_admin,
                new_admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
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
        require_admin(&env)?;

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

        storage::update_event(&env, event_info.clone());
        // Private events are excluded from the global tickets sold counter.
        if !event_info.is_private {
            storage::add_to_global_tickets_sold(&env, quantity_i128);
        }

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
        storage::update_event(&env, event_info.clone());
        // Private events are excluded from the global tickets sold counter.
        if !event_info.is_private {
            storage::subtract_from_global_tickets_sold(&env, 1);
        }

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
        require_admin(&env)?;

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
        let admin = require_admin(&env)?;

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
        let admin = require_admin(&env)?;

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
        let admin = require_admin(&env)?;

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

        if matches!(event_info.status, EventStatus::Cancelled) {
            return Err(EventRegistryError::EventCancelled);
        }

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
        let organizer = Self::get_organizer_address(env.clone(), event_id.clone())
            .ok_or(EventRegistryError::EventNotFound)?;
        organizer.require_auth();

        storage::authorize_scanner(&env, event_id.clone(), &scanner);

        env.events().publish(
            (AgoraEvent::ScannerAuthorized,),
            ScannerAuthorizedEvent {
                event_id,
                scanner,
                authorized_by: organizer,
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
        require_admin(&env)?;

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
    /// * `loyalty_multiplier` - Points multiplier from the purchased tier (0 or 1 = 1x, 2 = 2x, etc.)
    pub fn update_loyalty_score(
        env: Env,
        caller: Address,
        guest: Address,
        tickets_purchased: u32,
        amount_spent: i128,
        loyalty_multiplier: u32,
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

        // Award 10 points per ticket, scaled by the tier's loyalty_multiplier.
        // A multiplier of 0 is treated as 1x to avoid zeroing out points.
        let effective_multiplier = if loyalty_multiplier == 0 {
            1u64
        } else {
            loyalty_multiplier as u64
        };
        let points_earned = (tickets_purchased as u64)
            .saturating_mul(10)
            .saturating_mul(effective_multiplier);
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

    // ── Governance / Multi-Sig ─────────────────────────────────────────────────

    /// Returns the current multi-sig configuration
    pub fn get_multisig_config(env: Env) -> MultiSigConfig {
        storage::get_multisig_config(&env).unwrap_or_else(|| {
            let admins = Vec::new(&env);
            MultiSigConfig {
                admins,
                threshold: 1,
            }
        })
    }

    /// Checks if an address is an admin
    pub fn is_admin(env: Env, address: Address) -> bool {
        if let Some(config) = storage::get_multisig_config(&env) {
            config.admins.contains(&address)
        } else {
            false
        }
    }

    /// Sets the multi-sig configuration directly. Only callable by an existing admin.
    ///
    /// Validates that:
    /// - `new_admins` is not empty
    /// - All addresses in `new_admins` are valid
    /// - `new_threshold` is at least 1
    /// - `new_threshold` is not greater than the total number of admins
    ///
    /// # Arguments
    /// * `caller` - An existing admin authorizing this change
    /// * `new_admins` - The new list of admin addresses
    /// * `new_threshold` - The new approval threshold
    pub fn set_multisig_config(
        env: Env,
        caller: Address,
        new_admins: Vec<Address>,
        new_threshold: u32,
    ) -> Result<(), EventRegistryError> {
        caller.require_auth();

        let config =
            storage::get_multisig_config(&env).ok_or(EventRegistryError::NotInitialized)?;

        if !config.admins.contains(&caller) {
            return Err(EventRegistryError::Unauthorized);
        }

        if new_admins.is_empty() {
            return Err(EventRegistryError::CannotRemoveLastAdmin);
        }

        for addr in new_admins.iter() {
            validate_address(&env, &addr)?;
        }

        if new_threshold == 0 || new_threshold > new_admins.len() {
            return Err(EventRegistryError::InvalidThreshold);
        }

        let multisig_config = MultiSigConfig {
            admins: new_admins,
            threshold: new_threshold,
        };

        storage::set_multisig_config(&env, &multisig_config);

        Ok(())
    }

    /// Proposes a parameter change. Only callable by an existing admin.
    /// The proposer automatically approves the proposal.
    ///
    /// # Arguments
    /// * `proposer` - Admin address creating the proposal
    /// * `change` - The parameter change to propose
    /// * `expiry_ledgers` - Number of ledgers until proposal expires (0 = default 100800 ledgers ~7 days)
    pub fn propose_parameter_change(
        env: Env,
        proposer: Address,
        change: types::ParameterChange,
        expiry_ledgers: u64,
    ) -> Result<u64, EventRegistryError> {
        proposer.require_auth();

        // Verify proposer is an admin
        let config =
            storage::get_multisig_config(&env).ok_or(EventRegistryError::NotInitialized)?;

        if !config.admins.contains(&proposer) {
            return Err(EventRegistryError::Unauthorized);
        }

        // Validate the proposed change
        match &change {
            types::ParameterChange::AddAdmin(addr) => {
                validate_address(&env, addr)?;
                if config.admins.contains(addr) {
                    return Err(EventRegistryError::AdminAlreadyExists);
                }
            }
            types::ParameterChange::RemoveAdmin(addr) => {
                if !config.admins.contains(addr) {
                    return Err(EventRegistryError::AdminNotFound);
                }
                // Ensure we don't remove the last admin
                if config.admins.len() <= 1 {
                    return Err(EventRegistryError::CannotRemoveLastAdmin);
                }
            }
            types::ParameterChange::SetThreshold(threshold) => {
                if *threshold == 0 {
                    return Err(EventRegistryError::InvalidThreshold);
                }
                if *threshold > config.admins.len() {
                    return Err(EventRegistryError::InvalidThreshold);
                }
            }
            types::ParameterChange::UpdatePlatformWallet(addr) => {
                validate_address(&env, addr)?;
            }
            types::ParameterChange::SetPlatformFee(fee) => {
                if *fee > 10000 {
                    return Err(EventRegistryError::InvalidFeePercent);
                }
            }
            types::ParameterChange::SetMinStakeAmount(amount) => {
                if *amount <= 0 {
                    return Err(EventRegistryError::InvalidStakeAmount);
                }
            }
        }

        // Create proposal
        let proposal_id = storage::get_proposal_counter(&env);
        storage::set_proposal_counter(&env, proposal_id + 1);

        let default_expiry = 100800u64; // ~7 days at 5s per ledger
        let expiry = if expiry_ledgers == 0 {
            default_expiry
        } else {
            expiry_ledgers
        };

        let mut approvals = Vec::new(&env);
        approvals.push_back(proposer.clone());

        let proposal = types::Proposal {
            proposal_id,
            proposer: proposer.clone(),
            change,
            approvals,
            executed: false,
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + expiry,
        };

        storage::set_proposal(&env, &proposal);
        storage::add_active_proposal(&env, proposal_id);

        Ok(proposal_id)
    }

    /// Convenience function to propose adding an admin
    pub fn propose_add_admin(
        env: Env,
        proposer: Address,
        new_admin: Address,
        expiry_ledgers: u64,
    ) -> Result<u64, EventRegistryError> {
        Self::propose_parameter_change(
            env,
            proposer,
            types::ParameterChange::AddAdmin(new_admin),
            expiry_ledgers,
        )
    }

    /// Convenience function to propose removing an admin
    pub fn propose_remove_admin(
        env: Env,
        proposer: Address,
        admin_to_remove: Address,
        expiry_ledgers: u64,
    ) -> Result<u64, EventRegistryError> {
        Self::propose_parameter_change(
            env,
            proposer,
            types::ParameterChange::RemoveAdmin(admin_to_remove),
            expiry_ledgers,
        )
    }

    /// Convenience function to propose setting the threshold
    pub fn propose_set_threshold(
        env: Env,
        proposer: Address,
        new_threshold: u32,
        expiry_ledgers: u64,
    ) -> Result<u64, EventRegistryError> {
        Self::propose_parameter_change(
            env,
            proposer,
            types::ParameterChange::SetThreshold(new_threshold),
            expiry_ledgers,
        )
    }

    /// Convenience function to propose updating the platform wallet
    pub fn propose_set_platform_wallet(
        env: Env,
        proposer: Address,
        new_wallet: Address,
        expiry_ledgers: u64,
    ) -> Result<u64, EventRegistryError> {
        Self::propose_parameter_change(
            env,
            proposer,
            types::ParameterChange::UpdatePlatformWallet(new_wallet),
            expiry_ledgers,
        )
    }

    /// Convenience function to propose updating the platform fee
    pub fn propose_set_platform_fee(
        env: Env,
        proposer: Address,
        new_fee_percent: u32,
        expiry_ledgers: u64,
    ) -> Result<u64, EventRegistryError> {
        Self::propose_parameter_change(
            env,
            proposer,
            types::ParameterChange::SetPlatformFee(new_fee_percent),
            expiry_ledgers,
        )
    }

    /// Convenience function to propose updating the minimum stake amount
    pub fn propose_set_min_stake_amount(
        env: Env,
        proposer: Address,
        new_min_amount: i128,
        expiry_ledgers: u64,
    ) -> Result<u64, EventRegistryError> {
        Self::propose_parameter_change(
            env,
            proposer,
            types::ParameterChange::SetMinStakeAmount(new_min_amount),
            expiry_ledgers,
        )
    }

    /// Approves a proposal. Only callable by an admin.
    pub fn approve_proposal(
        env: Env,
        approver: Address,
        proposal_id: u64,
    ) -> Result<(), EventRegistryError> {
        approver.require_auth();

        // Verify approver is an admin
        let config =
            storage::get_multisig_config(&env).ok_or(EventRegistryError::NotInitialized)?;

        if !config.admins.contains(&approver) {
            return Err(EventRegistryError::Unauthorized);
        }

        // Get proposal
        let mut proposal =
            storage::get_proposal(&env, proposal_id).ok_or(EventRegistryError::ProposalNotFound)?;

        // Check if already executed
        if proposal.executed {
            return Err(EventRegistryError::ProposalAlreadyExecuted);
        }

        // Check if expired
        if env.ledger().timestamp() > proposal.expires_at {
            return Err(EventRegistryError::ProposalExpired);
        }

        // Check if already approved by this admin
        if proposal.approvals.contains(&approver) {
            return Err(EventRegistryError::AlreadyApproved);
        }

        // Add approval
        proposal.approvals.push_back(approver);
        storage::set_proposal(&env, &proposal);

        Ok(())
    }

    /// Executes a proposal if it has met the approval threshold.
    /// Only callable by an admin.
    pub fn execute_proposal(
        env: Env,
        executor: Address,
        proposal_id: u64,
    ) -> Result<(), EventRegistryError> {
        executor.require_auth();

        // Verify executor is an admin
        let config =
            storage::get_multisig_config(&env).ok_or(EventRegistryError::NotInitialized)?;

        if !config.admins.contains(&executor) {
            return Err(EventRegistryError::Unauthorized);
        }

        // Get proposal
        let mut proposal =
            storage::get_proposal(&env, proposal_id).ok_or(EventRegistryError::ProposalNotFound)?;

        // Check if already executed
        if proposal.executed {
            return Err(EventRegistryError::ProposalAlreadyExecuted);
        }

        // Check if expired
        if env.ledger().timestamp() > proposal.expires_at {
            return Err(EventRegistryError::ProposalExpired);
        }

        // Check if threshold is met
        if proposal.approvals.len() < config.threshold {
            return Err(EventRegistryError::InsufficientApprovals);
        }

        // Execute the proposal
        match &proposal.change {
            types::ParameterChange::AddAdmin(new_admin) => {
                let mut new_config = config.clone();
                new_config.admins.push_back(new_admin.clone());
                storage::set_multisig_config(&env, &new_config);
                storage::set_admin(&env, new_admin); // Update legacy admin storage
            }
            types::ParameterChange::RemoveAdmin(admin_to_remove) => {
                let mut new_config = config.clone();
                let mut new_admins = Vec::new(&env);
                for admin in new_config.admins.iter() {
                    if admin != *admin_to_remove {
                        new_admins.push_back(admin);
                    }
                }
                new_config.admins = new_admins;

                // Adjust threshold if necessary
                if new_config.threshold > new_config.admins.len() {
                    new_config.threshold = new_config.admins.len();
                }

                storage::set_multisig_config(&env, &new_config);
            }
            types::ParameterChange::SetThreshold(new_threshold) => {
                let mut new_config = config.clone();
                new_config.threshold = *new_threshold;
                storage::set_multisig_config(&env, &new_config);
            }
            types::ParameterChange::UpdatePlatformWallet(new_wallet) => {
                storage::set_platform_wallet(&env, new_wallet);
            }
            types::ParameterChange::SetPlatformFee(new_fee) => {
                storage::set_platform_fee(&env, *new_fee);
                env.events().publish(
                    (AgoraEvent::FeeUpdated,),
                    FeeUpdatedEvent {
                        new_fee_percent: *new_fee,
                    },
                );
            }
            types::ParameterChange::SetMinStakeAmount(new_amount) => {
                storage::set_min_stake_amount(&env, *new_amount);
            }
        }

        // Mark as executed
        proposal.executed = true;
        storage::set_proposal(&env, &proposal);
        storage::remove_active_proposal(&env, proposal_id);

        Ok(())
    }

    /// Gets a proposal by ID
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<types::Proposal> {
        storage::get_proposal(&env, proposal_id)
    }

    /// Gets all active proposal IDs
    pub fn get_active_proposals(env: Env) -> Vec<u64> {
        storage::get_active_proposals(&env)
    }

    /// Returns the contract version.
    pub fn version(_env: Env) -> u32 {
        VERSION
    }
}

fn require_admin(env: &Env) -> Result<Address, EventRegistryError> {
    let admin = storage::get_admin(env).ok_or(EventRegistryError::NotInitialized)?;
    admin.require_auth();
    Ok(admin)
}

fn validate_address(env: &Env, address: &Address) -> Result<(), EventRegistryError> {
    if address == &env.current_contract_address() || is_zero_address(env, address) {
        return Err(EventRegistryError::InvalidAddress);
    }
    Ok(())
}

fn is_zero_address(env: &Env, address: &Address) -> bool {
    let zero_account = String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJXFF",
    );
    address.to_string() == zero_account
}

/// Trims leading and trailing ASCII whitespace from a Soroban `String`.
fn trim_string(env: &Env, s: &String) -> String {
    let len = s.len();
    if len == 0 {
        return s.clone();
    }

    // Copy string bytes into a stack buffer for manipulation
    let mut buf = [0u8; 256];
    let len_usize = len as usize;
    // If name exceeds buffer, return as-is (shouldn't happen in practice)
    if len_usize > buf.len() {
        return s.clone();
    }
    s.copy_into_slice(&mut buf[..len_usize]);

    let mut start = 0usize;
    while start < len_usize {
        let b = buf[start];
        if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
            start += 1;
        } else {
            break;
        }
    }

    let mut end = len_usize;
    while end > start {
        let b = buf[end - 1];
        if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
            end -= 1;
        } else {
            break;
        }
    }

    if start == 0 && end == len_usize {
        return s.clone();
    }

    String::from_bytes(env, &buf[start..end])
}

fn validate_metadata_cid(env: &Env, cid: &String) -> Result<(), EventRegistryError> {
    let cid_len = cid.len();
    if !(MIN_METADATA_CID_LEN..=MAX_METADATA_CID_LEN).contains(&cid_len) {
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

fn validate_restocking_fee(args: &EventRegistrationArgs) -> Result<(), EventRegistryError> {
    if args.restocking_fee < 0 {
        return Err(EventRegistryError::InvalidFeeCalculation);
    }

    if args.restocking_fee == 0 {
        return Ok(());
    }

    for tier in args.tiers.values() {
        let remaining_refund = tier
            .price
            .checked_sub(args.restocking_fee)
            .ok_or(EventRegistryError::InvalidFeeCalculation)?;

        if remaining_refund < 0 {
            return Err(EventRegistryError::RestockingFeeExceedsTicketPrice);
        }
    }

    Ok(())
}

/// Suspends all active events for a blacklisted organizer.
/// This implements the "Suspension" ripple effect.
#[allow(deprecated)]
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

#[cfg(test)]
mod test_multisig;

#[cfg(test)]
mod test_free_ticket;
