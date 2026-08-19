#![no_std]

use crate::events::{
    AgoraEvent, CollateralStakedEvent, CollateralUnstakedEvent, CustomFeeSetEvent,
    DisputeOpenedEvent, DisputeResolvedEvent, DisputeVotedEvent, EventArchivedEvent,
    EventCancelledEvent, EventPostponedEvent, EventRegisteredEvent,
    EventStatusUpdatedEvent, EventsSuspendedEvent, FeeUpdatedEvent, FeedbackCidSetEvent,
    GlobalPromoUpdatedEvent, GoalMetEvent, InitializationEvent, InventoryIncrementedEvent,
    LoyaltyScoreUpdatedEvent, MetadataUpdatedEvent, MinStakeAmountUpdatedEvent,
    OrganizerBlacklistedEvent, OrganizerRemovedFromBlacklistEvent, ProposalCancelledEvent,
    RegistryUpgradedEvent, ScannerAuthorizedEvent, ScannerRevokedEvent, StakerRewardsClaimedEvent,
    StakerRewardsDistributedEvent, StakingTokenUpdatedEvent, WaitlistJoinedEvent,
    WaitlistLeftEvent,
};
use crate::types::{
    BlacklistAuditEntry, EventInfo, EventReceipt, EventRegistrationArgs, EventStatus, GuestProfile,
    MultiSigConfig, OrganizerStake, PaymentInfo, TicketTier,
};
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, String, Vec};

mod auth;
pub mod dispute;
pub mod error;
pub mod events;
pub mod storage;
mod topics;
pub mod types;

use crate::types::{SeriesPass, SeriesRegistry};

use crate::error::EventRegistryError;

/// Maximum number of ticket tiers allowed per event during registration.
const MAX_TIERS_PER_EVENT: u32 = 20;

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
                return Err(EventRegistryError::Unauthorized);
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

    /// Check if a holder has a valid pass for a given series.
    /// Returns true only if a pass exists, usage limit has not been reached, and the pass has not expired.
    pub fn has_valid_series_pass(env: Env, holder: Address, series_id: String) -> bool {
        if let Some(pass) = storage::get_holder_series_pass(&env, &holder, series_id) {
            if pass.usage_limit > 0 && pass.usage_count >= pass.usage_limit {
                return false;
            }
            if pass.expires_at > 0 && env.ledger().timestamp() >= pass.expires_at {
                return false;
            }
            true
        } else {
            false
        }
    }

    /// Records a redemption/check-in against a series pass, incrementing its
    /// usage count. Returns `EventNotFound` if the pass doesn't exist or its
    /// usage limit has already been reached (mirrors `storage::increment_series_pass_usage`,
    /// which was previously only reachable internally, not as a contract call).
    pub fn increment_series_pass_usage(
        env: Env,
        pass_id: String,
    ) -> Result<(), EventRegistryError> {
        storage::increment_series_pass_usage(&env, pass_id)
            .map(|_| ())
            .ok_or(EventRegistryError::EventNotFound)
    }

    /// Initializes the contract configuration. Can only be called once.
    /// Sets up initial admin with multi-sig configuration (threshold = 1 for single admin).
    /// The `usdc_token` address is automatically added to the payment token whitelist.
    ///
    /// # Arguments
    /// * `admin` - The administrator address.
    /// * `platform_wallet` - The platform wallet address for fees.
    /// * `platform_fee_percent` - Initial platform fee in basis points (0–10000; 10000 = 100%).
    ///   An explicit `0` sets a zero platform fee (no forced default).
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

        // Valid range is 0–10000 basis points inclusive. An explicit 0 means
        // zero platform fee (no forced default).
        if platform_fee_percent > 10000 {
            return Err(EventRegistryError::InvalidFeePercent);
        }
        let initial_fee = platform_fee_percent;

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
        let _admin = auth::require_admin(&env)?;
        validate_address(&env, &token)?;
        storage::add_to_token_whitelist(&env, &token);
        Ok(())
    }

    /// Removes a token address from the payment token whitelist. Only callable by the administrator.
    pub fn remove_from_token_whitelist(env: Env, token: Address) -> Result<(), EventRegistryError> {
        let _admin = auth::require_admin(&env)?;
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
        args.organizer_address.require_auth();

        // Check if organizer is blacklisted
        if storage::is_blacklisted(&env, &args.organizer_address) {
            return Err(EventRegistryError::OrganizerBlacklisted);
        }

        validate_metadata_cid(&env, &args.metadata_cid)?;

        // Validate tags if provided
        if let Some(ref tags) = args.tags {
            validate_tags(&env, tags)?;
        }

        if storage::event_exists(&env, args.event_id.clone()) {
            return Err(EventRegistryError::EventAlreadyExists);
        }

        if args.tiers.len() > MAX_TIERS_PER_EVENT {
            return Err(EventRegistryError::TooManyTiers);
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
                return Err(EventRegistryError::TierLimitExceeds);
            }
        }

        // Validate resale cap if provided
        if let Some(cap) = args.resale_cap_bps {
            if cap > 10000 {
                return Err(EventRegistryError::InvalidResaleCapBps);
            }
        }

        // Validate milestone plan: the sum of all release_percent values
        // (basis points) must not exceed 10000 (100%), or the plan would
        // release more revenue than was ever collected (Issue #850).
        if let Some(ref milestones) = args.milestone_plan {
            let mut total_release_bps: u32 = 0;
            for milestone in milestones.iter() {
                total_release_bps = total_release_bps
                    .checked_add(milestone.release_percent)
                    .ok_or(EventRegistryError::InvalidMilestonePlan)?;
            }
            if total_release_bps > 10000 {
                return Err(EventRegistryError::InvalidMilestonePlan);
            }
        }

        // Validate event time range
        if args.start_time != 0 && args.end_time != 0 && args.end_time <= args.start_time {
            return Err(EventRegistryError::InvalidDeadline);
        }

        let platform_fee_percent = storage::get_platform_fee(&env);

        let event_info = EventInfo {
            event_id: args.event_id.clone(),
            name: args.name.clone(),
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
            category_ids: args.category_ids,
            start_time: args.start_time,
            is_private: args.is_private,
            end_time: args.end_time,
            transfer_lock_duration: args.transfer_lock_duration,
            accepted_tokens: args.accepted_tokens,
            use_global_whitelist: args.use_global_whitelist,
            feedback_cid: None,
            cancellation_reason: None,
            referral_rate_bps: args.referral_rate_bps.unwrap_or(0),
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
                    referral_rate_bps: event_info.referral_rate_bps,
                })
            }
            None => Err(EventRegistryError::EventNotFound),
        }
    }

    /// Returns the cumulative number of events ever registered on the platform.
    pub fn get_global_event_count(env: Env) -> u32 {
        storage::get_global_event_count(&env)
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
                auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

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
    pub fn cancel_event(
        env: Env,
        event_id: String,
        reason: Option<String>,
    ) -> Result<(), EventRegistryError> {
        match storage::get_event(&env, event_id.clone()) {
            Some(mut event_info) => {
                // Verify organizer signature
                auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

                if matches!(event_info.status, EventStatus::Cancelled) {
                    return Err(EventRegistryError::EventAlreadyCanceled);
                }

                // Update status to Cancelled and deactivate
                event_info.status = EventStatus::Cancelled;
                event_info.is_active = false;
                event_info.cancellation_reason = reason.clone();
                storage::update_event(&env, event_info.clone());

                // Emit cancellation event
                env.events().publish(
                    (AgoraEvent::EventCancelled,),
                    EventCancelledEvent {
                        event_id,
                        cancelled_by: event_info.organizer_address,
                        timestamp: env.ledger().timestamp(),
                        reason,
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
                auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

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
                auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

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

    /// Sets the post-event feedback IPFS CID. Only callable by the organizer after end_time.
    pub fn set_feedback_cid(
        env: Env,
        event_id: String,
        feedback_cid: String,
    ) -> Result<(), EventRegistryError> {
        match storage::get_event(&env, event_id.clone()) {
            Some(mut event_info) => {
                auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

                require_event_ended(&env, &event_info)?;

                validate_metadata_cid(&env, &feedback_cid)?;

                if event_info.feedback_cid.as_ref() == Some(&feedback_cid) {
                    return Ok(());
                }

                event_info.feedback_cid = Some(feedback_cid.clone());
                storage::update_event(&env, event_info.clone());

                env.events().publish(
                    (AgoraEvent::FeedbackCidSet,),
                    FeedbackCidSetEvent {
                        event_id,
                        feedback_cid,
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
        auth::require_organizer(&env, &event_info.event_id, &event_info.organizer_address).unwrap();
        if event_info.feedback_cid.is_some() {
            require_event_ended(&env, &event_info).unwrap();
        }
        // Validate resale_cap_bps on every write path (Issue #883).
        if let Some(cap) = event_info.resale_cap_bps {
            if cap > 10000 {
                panic!("InvalidResaleCapBps");
            }
        }
        storage::store_event(&env, event_info);
    }

    /// Updates the resale cap for an event. Only callable by the event organizer.
    ///
    /// # Arguments
    /// * `event_id` - The event to update.
    /// * `resale_cap_bps` - New resale cap in basis points (`None` removes the cap).
    ///   Must be `<= 10000` when `Some`.
    pub fn set_resale_cap_bps(
        env: Env,
        event_id: String,
        resale_cap_bps: Option<u32>,
    ) -> Result<(), EventRegistryError> {
        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

        if let Some(cap) = resale_cap_bps {
            if cap > 10000 {
                return Err(EventRegistryError::InvalidResaleCapBps);
            }
        }

        event_info.resale_cap_bps = resale_cap_bps;
        storage::update_event(&env, event_info);

        Ok(())
    }

    /// Retrieves an event by its ID.
    pub fn get_event(env: Env, event_id: String) -> Option<EventInfo> {
        storage::get_event(&env, event_id)
    }

    /// Retrieves a batch of events by their IDs (max 50).
    pub fn get_events_batch(
        env: Env,
        event_ids: Vec<String>,
    ) -> Result<Vec<Option<EventInfo>>, EventRegistryError> {
        if event_ids.len() > 50 {
            return Err(EventRegistryError::TooManyTiers);
        }
        let mut results = Vec::new(&env);
        for id in event_ids.into_iter() {
            results.push_back(storage::get_event(&env, id));
        }
        Ok(results)
    }

    /// Returns the total number of tickets sold across all events.
    pub fn get_global_tickets_sold(env: Env) -> i128 {
        storage::get_global_tickets_sold(&env)
    }

    /// Checks if an event exists.
    pub fn event_exists(env: Env, event_id: String) -> bool {
        storage::event_exists(&env, event_id)
    }

    /// Retrieves all event IDs for an organizer.
    /// If the caller is not the organizer, private events are filtered out (Issue #880).
    pub fn get_organizer_events(env: Env, organizer: Address, caller: Address) -> Vec<String> {
        storage::get_organizer_events(&env, &organizer, &caller)
    }

    /// Updates the platform fee percentage. Only callable by the administrator.
    pub fn set_platform_fee(env: Env, new_fee_percent: u32) -> Result<(), EventRegistryError> {
        let _admin = auth::require_admin(&env)?;

        if new_fee_percent > 10000 {
            return Err(EventRegistryError::InvalidFeePercent);
        }

        // When threshold > 1 (multi-sig), a pre-approved SetPlatformFee proposal is required.
        let config =
            storage::get_multisig_config(&env).ok_or(EventRegistryError::NotInitialized)?;
        if config.threshold > 1 {
            // Find an approved, unexpired, unexecuted SetPlatformFee proposal matching new_fee_percent
            let active = storage::get_active_proposals(&env);
            let mut approved_proposal_id: Option<u64> = None;
            let now = env.ledger().timestamp();
            for pid in active.iter() {
                if let Some(p) = storage::get_proposal(&env, pid) {
                    if p.executed || p.cancelled || now > p.expires_at {
                        continue;
                    }
                    if let types::ParameterChange::SetPlatformFee(fee) = &p.change {
                        if *fee == new_fee_percent && p.approvals.len() >= config.threshold {
                            approved_proposal_id = Some(pid);
                            break;
                        }
                    }
                }
            }
            let proposal_id = approved_proposal_id.ok_or(EventRegistryError::MultisigError)?;
            // Mark the proposal as executed
            let mut proposal = storage::get_proposal(&env, proposal_id).unwrap();
            proposal.executed = true;
            storage::set_proposal(&env, &proposal);
            storage::remove_active_proposal(&env, proposal_id);
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
        let admin = auth::require_admin(&env)?;

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
        let _admin = auth::require_admin(&env)?;

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
    /// * `TokenNotAccepted` - If the event configured a non-empty `accepted_tokens`
    ///   list and `payment_token` is not in it (Issue #851). This is a defense-in-depth
    ///   cross-validation: the primary enforcement lives in the `TicketPayment`
    ///   contract, but a misconfigured or malicious caller of this function
    ///   would otherwise bypass it entirely.
    pub fn increment_inventory(
        env: Env,
        event_id: String,
        tier_id: String,
        user: Address,
        quantity: u32,
        payment_token: Address,
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

        // Issue #851: when the event restricts payments to specific tokens,
        // enforce that here too rather than trusting the caller (normally
        // the TicketPayment contract) to have already validated it.
        if !event_info.accepted_tokens.is_empty()
            && !event_info.accepted_tokens.contains(&payment_token)
        {
            return Err(EventRegistryError::TokenNotAccepted);
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
            return Err(EventRegistryError::TierSoldOut);
        }

        // Per-user limit enforcement
        if tier.max_per_user > 0 {
            let user_count = storage::get_user_ticket_count(&env, &event_id, &tier_id, &user);
            let new_user_count = user_count
                .checked_add(quantity)
                .ok_or(EventRegistryError::SupplyOverflow)?;
            if new_user_count > tier.max_per_user {
                return Err(EventRegistryError::PerUserLimitExceeded);
            }
        }

        tier.current_sold = new_tier_sold;
        event_info.tiers.set(tier_id.clone(), tier.clone());

        event_info.current_supply = event_info
            .current_supply
            .checked_add(quantity_i128)
            .ok_or(EventRegistryError::SupplyOverflow)?;

        let new_supply = event_info.current_supply;

        // Update per-user ticket count after all checks pass
        if tier.max_per_user > 0 {
            storage::add_to_user_ticket_count(&env, &event_id, &tier_id, &user, quantity);
        }

        // Update global tickets sold counter
        storage::add_to_global_tickets_sold(&env, quantity_i128);

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

    /// Checks whether a specific tier for an event is sold out.
    pub fn is_tier_sold_out(
        env: Env,
        event_id: String,
        tier_id: String,
    ) -> Result<bool, EventRegistryError> {
        let event_info =
            storage::get_event(&env, event_id).ok_or(EventRegistryError::EventNotFound)?;
        let tier = event_info
            .tiers
            .get(tier_id)
            .ok_or(EventRegistryError::TierNotFound)?;
        Ok(tier.current_sold >= tier.tier_limit)
    }

    /// Adds a new ticket tier to an existing event.
    /// Restricted to the event organizer.
    pub fn add_tier(
        env: Env,
        event_id: String,
        tier_id: String,
        tier: TicketTier,
    ) -> Result<(), EventRegistryError> {
        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;
        event_info.organizer_address.require_auth();

        if event_info.max_supply > 0 {
            let mut total_tier_limit: i128 = 0;
            for existing_tier in event_info.tiers.values() {
                total_tier_limit = total_tier_limit
                    .checked_add(existing_tier.tier_limit)
                    .ok_or(EventRegistryError::SupplyOverflow)?;
            }
            total_tier_limit = total_tier_limit
                .checked_add(tier.tier_limit)
                .ok_or(EventRegistryError::SupplyOverflow)?;

            if total_tier_limit > event_info.max_supply {
                return Err(EventRegistryError::TierLimitExceeds);
            }
        }

        event_info.tiers.set(tier_id, tier);
        storage::update_event(&env, event_info);
        Ok(())
    }

    /// Deactivates a tier by setting its limit equal to current_sold, preventing further sales.
    /// Restricted to the event organizer.
    pub fn deactivate_tier(
        env: Env,
        event_id: String,
        tier_id: String,
    ) -> Result<(), EventRegistryError> {
        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;
        event_info.organizer_address.require_auth();

        let mut tier = event_info
            .tiers
            .get(tier_id.clone())
            .ok_or(EventRegistryError::TierNotFound)?;

        tier.tier_limit = tier.current_sold;
        event_info.tiers.set(tier_id, tier);
        storage::update_event(&env, event_info);
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
        let _admin = auth::require_admin(&env)?;

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
        let admin = auth::require_admin(&env)?;

        validate_address(&env, &organizer_address)?;

        // Check if already blacklisted
        if storage::is_blacklisted(&env, &organizer_address) {
            return Ok(());
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
        let admin = auth::require_admin(&env)?;

        validate_address(&env, &organizer_address)?;

        // Check if currently blacklisted
        if !storage::is_blacklisted(&env, &organizer_address) {
            return Err(EventRegistryError::OrgNotBlacklisted);
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
        let admin = auth::require_admin(&env)?;

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
        let expiry = storage::get_promo_expiry(&env);
        if expiry <= env.ledger().timestamp() {
            return 0;
        }

        storage::get_global_promo_bps(&env)
    }

    /// Returns the active global promotional discount and expiry timestamp.
    pub fn get_global_promo(env: Env) -> Option<(u32, u64)> {
        let expiry = storage::get_promo_expiry(&env);
        if expiry <= env.ledger().timestamp() {
            return None;
        }

        Some((storage::get_global_promo_bps(&env), expiry))
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
        new_start_time: u64,
        grace_period_end: u64,
    ) -> Result<(), EventRegistryError> {
        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        // Only the organizer may postpone their event.
        auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

        let now = env.ledger().timestamp();
        if new_start_time <= now {
            return Err(EventRegistryError::InvalidDeadline);
        }
        if grace_period_end <= now {
            return Err(EventRegistryError::InvalidGracePeriod);
        }

        event_info.start_time = new_start_time;
        event_info.is_postponed = true;
        event_info.grace_period_end = grace_period_end;
        storage::update_event(&env, event_info.clone());

        env.events().publish(
            (AgoraEvent::EventPostponed,),
            EventPostponedEvent {
                event_id,
                organizer_address: event_info.organizer_address,
                new_start_time,
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
        auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

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

    /// Revokes a previously authorized scanner wallet for a specific event.
    /// Only callable by the event organizer.
    pub fn revoke_scanner(
        env: Env,
        event_id: String,
        scanner: Address,
    ) -> Result<(), EventRegistryError> {
        let event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        // Only the organizer can revoke scanners
        auth::require_organizer(&env, &event_id, &event_info.organizer_address)?;

        storage::remove_scanner(&env, event_id.clone(), &scanner);

        env.events().publish(
            (AgoraEvent::ScannerRevoked,),
            ScannerRevokedEvent {
                event_id,
                scanner,
                revoked_by: event_info.organizer_address,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
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
        let admin = auth::require_admin(&env)?;

        if min_amount <= 0 {
            return Err(EventRegistryError::InvalidStakeAmount);
        }

        let old_token = storage::get_staking_token(&env);
        let old_amount = storage::get_min_stake_amount(&env);

        storage::set_staking_token(&env, &token);
        storage::set_min_stake_amount(&env, min_amount);

        env.events().publish(
            (AgoraEvent::StakingTokenUpdated,),
            StakingTokenUpdatedEvent {
                old_token,
                new_token: token,
                admin: admin.clone(),
            },
        );

        env.events().publish(
            (AgoraEvent::MinStakeAmountUpdated,),
            MinStakeAmountUpdatedEvent {
                old_amount,
                new_amount: min_amount,
                admin,
            },
        );

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
    /// * `loyalty_multiplier` - Tier multiplier for loyalty points; 0 is treated as 1x
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

        // Award 10 base points per ticket, adjusted by the tier multiplier.
        let effective_multiplier = if loyalty_multiplier == 0 {
            1
        } else {
            loyalty_multiplier
        } as u64;
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
                    return Err(EventRegistryError::Unauthorized);
                }
                // Ensure we don't remove the last admin
                if config.admins.len() <= 1 {
                    return Err(EventRegistryError::CannotRemoveLast);
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
            cancelled: false,
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
            storage::get_proposal(&env, proposal_id).ok_or(EventRegistryError::MultisigError)?;

        // Check if expired
        if env.ledger().timestamp() > proposal.expires_at {
            return Err(EventRegistryError::ProposalExpired);
        }

        // Check if already executed
        if proposal.executed {
            return Err(EventRegistryError::PropAlreadyExecuted);
        }

        // Check if already cancelled
        if proposal.cancelled {
            return Err(EventRegistryError::PropAlreadyCanceled);
        }

        // Check if already approved by this admin
        if proposal.approvals.contains(&approver) {
            return Ok(()); // Already approved, no-op
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
            storage::get_proposal(&env, proposal_id).ok_or(EventRegistryError::MultisigError)?;

        // Check if already executed
        if proposal.executed {
            return Err(EventRegistryError::PropAlreadyExecuted);
        }

        // Check if already cancelled
        if proposal.cancelled {
            return Err(EventRegistryError::PropAlreadyCanceled);
        }

        // Check if expired
        if env.ledger().timestamp() > proposal.expires_at {
            return Err(EventRegistryError::ProposalExpired);
        }

        // Check if threshold is met
        if proposal.approvals.len() < config.threshold {
            return Err(EventRegistryError::MultisigError);
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
                    if admin != admin_to_remove.clone() {
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
            types::ParameterChange::SetPlatformFee(fee) => {
                storage::set_platform_fee(&env, *fee);
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

    /// Removes expired proposals from the `ActiveProposals` list.
    ///
    /// Any proposal whose `expires_at` timestamp has passed is considered expired and
    /// will be removed from the active list. This prevents unbounded growth of the
    /// list over time. Any admin may call this function.
    ///
    /// # Returns
    /// The number of expired proposals that were removed.
    pub fn cleanup_expired_proposals(env: Env) -> Result<u32, EventRegistryError> {
        let _admin = auth::require_admin(&env)?;

        let now = env.ledger().timestamp();
        let active = storage::get_active_proposals(&env);
        let mut removed: u32 = 0;

        for proposal_id in active.iter() {
            if let Some(proposal) = storage::get_proposal(&env, proposal_id) {
                if now > proposal.expires_at && !proposal.executed && !proposal.cancelled {
                    storage::remove_active_proposal(&env, proposal_id);
                    removed += 1;
                }
            }
        }

        Ok(removed)
    }

    /// Joins the waitlist for an event. Emits WaitlistJoinedEvent.
    /// Requires the user's authentication.
    pub fn join_waitlist(
        env: Env,
        event_id: String,
        user: Address,
    ) -> Result<(), EventRegistryError> {
        user.require_auth();

        // Check if event exists
        if !storage::event_exists(&env, event_id.clone()) {
            return Err(EventRegistryError::EventNotFound);
        }

        // Check if user is already on the waitlist
        if storage::is_on_waitlist(&env, &event_id, &user) {
            return Err(EventRegistryError::AlreadyOnWaitlist);
        }

        // Add user to waitlist
        storage::add_to_waitlist(&env, &event_id, &user);

        // Emit WaitlistJoinedEvent
        env.events().publish(
            (AgoraEvent::WaitlistJoined,),
            WaitlistJoinedEvent {
                event_id,
                user,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Leaves the waitlist for an event. Emits WaitlistLeftEvent.
    /// Requires the user's authentication.
    pub fn leave_waitlist(
        env: Env,
        event_id: String,
        user: Address,
    ) -> Result<(), EventRegistryError> {
        user.require_auth();

        // Check if event exists
        if !storage::event_exists(&env, event_id.clone()) {
            return Err(EventRegistryError::EventNotFound);
        }

        // Check if user is on the waitlist
        if !storage::is_on_waitlist(&env, &event_id, &user) {
            return Err(EventRegistryError::NotOnWaitlist);
        }

        // Remove user from waitlist
        storage::remove_from_waitlist(&env, &event_id, &user);

        // Emit WaitlistLeftEvent
        env.events().publish(
            (AgoraEvent::WaitlistLeft,),
            WaitlistLeftEvent {
                event_id,
                user,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Cancels a governance proposal. Only the proposer can cancel their own proposal.
    /// A proposal cannot be cancelled if it has already been executed.
    /// Emits ProposalCancelledEvent.
    pub fn cancel_proposal(
        env: Env,
        proposer: Address,
        proposal_id: u64,
    ) -> Result<(), EventRegistryError> {
        proposer.require_auth();

        // Get proposal
        let mut proposal =
            storage::get_proposal(&env, proposal_id).ok_or(EventRegistryError::MultisigError)?;

        // Check if proposer is the original proposer
        if proposal.proposer != proposer {
            return Err(EventRegistryError::Unauthorized);
        }

        // Check if already executed
        if proposal.executed {
            return Err(EventRegistryError::PropAlreadyExecuted);
        }

        // Check if already cancelled
        if proposal.cancelled {
            return Err(EventRegistryError::PropAlreadyCanceled);
        }

        // Mark as cancelled
        proposal.cancelled = true;
        storage::set_proposal(&env, &proposal);

        // Remove from active proposals list
        storage::remove_active_proposal(&env, proposal_id);

        // Emit ProposalCancelledEvent
        env.events().publish(
            (AgoraEvent::ProposalCancelled,),
            ProposalCancelledEvent {
                proposal_id,
                cancelled_by: proposer,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Convenience function to propose setting the platform fee
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

    // ── Dispute ───────────────────────────────────────────────────────────

    /// Opens a dispute on an event. Only callable by a ticket holder within 48h post-event.
    pub fn open_dispute(env: Env, event_id: String, opened_by: Address) -> Result<(), EventRegistryError> {
        dispute::open_dispute(&env, event_id, opened_by)
    }

    /// Casts a vote on an open dispute. One vote per address.
    pub fn vote_on_dispute(
        env: Env,
        event_id: String,
        voter: Address,
        vote: crate::types::DisputeVote,
    ) -> Result<(), EventRegistryError> {
        dispute::vote_on_dispute(&env, event_id, voter, vote)
    }

    /// Resolves a dispute after voting ends. Counts votes and determines outcome.
    pub fn resolve_dispute(env: Env, event_id: String) -> Result<crate::types::DisputeStatus, EventRegistryError> {
        dispute::resolve_dispute(&env, event_id)
    }

    /// Returns the dispute for an event, if one exists.
    pub fn get_dispute(env: Env, event_id: String) -> Option<crate::types::Dispute> {
        dispute::get_dispute(&env, event_id)
    }

    /// Returns all votes for a dispute.
    pub fn get_dispute_votes(env: Env, event_id: String) -> Vec<crate::types::DisputeVote> {
        dispute::get_dispute_votes(&env, event_id)
    }
}

fn validate_address(env: &Env, address: &Address) -> Result<(), EventRegistryError> {
    if address == &env.current_contract_address() {
        return Err(EventRegistryError::InvalidAddress);
    }
    Ok(())
}

fn validate_metadata_cid(env: &Env, cid: &String) -> Result<(), EventRegistryError> {
    let len = cid.len();
    let mut bytes = soroban_sdk::Bytes::new(env);
    bytes.append(&cid.clone().into());

    // CIDv0: starts with "Qm" and is at least 46 characters long
    if len >= 46 && bytes.len() >= 2 && bytes.get(0) == Some(b'Q') && bytes.get(1) == Some(b'm') {
        return Ok(());
    }

    // CIDv1: starts with "bafy" and is at least 59 characters long
    if len >= 59
        && bytes.len() >= 4
        && bytes.get(0) == Some(b'b')
        && bytes.get(1) == Some(b'a')
        && bytes.get(2) == Some(b'f')
        && bytes.get(3) == Some(b'y')
    {
        return Ok(());
    }

    Err(EventRegistryError::InvalidMetadataCid)
}

/// Validates event tags to ensure they contain only printable characters.
/// Each tag must be ≤ 32 characters and contain only printable ASCII (32-126)
/// or Unicode letters/numbers/spaces. Rejects control characters, null bytes,
/// and other non-printable sequences.
fn validate_tags(env: &Env, tags: &soroban_sdk::Vec<String>) -> Result<(), EventRegistryError> {
    for tag in tags.iter() {
        // Check length
        if tag.len() > 32 {
            return Err(EventRegistryError::InvalidTags);
        }

        // Convert to bytes for character validation
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&tag.into());

        // Check each byte for printable characters
        for i in 0..bytes.len() {
            if let Some(byte) = bytes.get(i) {
                // Reject null bytes and control characters (0x00-0x1F, 0x7F-0x9F)
                // Accept printable ASCII (0x20-0x7E) and extended UTF-8 sequences (≥ 0xC0)
                if byte < 0x20 || (0x7F..0xC0).contains(&byte) {
                    return Err(EventRegistryError::InvalidTags);
                }
            }
        }
    }

    Ok(())
}

fn require_event_ended(env: &Env, event_info: &EventInfo) -> Result<(), EventRegistryError> {
    let now = env.ledger().timestamp();
    if event_info.end_time == 0 || now <= event_info.end_time {
        return Err(EventRegistryError::EventNotEnded);
    }
    Ok(())
}

/// Suspends all active events for a blacklisted organizer.
/// This implements the "Suspension" ripple effect.
fn suspend_organizer_events(
    env: Env,
    organizer_address: Address,
) -> Result<(), EventRegistryError> {
    // Pass organizer as both organizer and caller since this is an internal operation
    let organizer_events =
        storage::get_organizer_events(&env, &organizer_address, &organizer_address);
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
mod issue_tests;

#[cfg(test)]
mod test_issue_fixes;

#[cfg(test)]
mod test_global_promo;

// The legacy monolithic test modules are stale against the current contract API.
// Keep default `cargo test -p event-registry` focused on compilable coverage.

// TODO: Uncomment when multisig functions are implemented
// #[cfg(test)]
// mod test_multisig;
