use crate::storage::{
    add_discount_hash, add_payment_to_buyer_index, add_to_active_escrow_by_token,
    add_to_active_escrow_total, add_to_daily_withdrawn_amount,
    add_to_total_fees_collected_by_token, add_to_total_volume_processed, add_token_to_whitelist,
    get_admin, get_bulk_refund_index, get_daily_withdrawn_amount, get_event_balance,
    get_event_payments, get_event_registry, get_highest_bid, get_oracle_address,
    get_partial_refund_index, get_partial_refund_percentage, get_payment, get_platform_wallet,
    get_proposal, get_slippage_bps, get_total_fees_collected_by_token, get_total_governors,
    get_ticket_payment_id, get_transfer_fee, get_usdc_token, get_withdrawal_cap,
    has_price_switched, increment_proposal_count, is_auction_closed, is_discount_hash_used,
    is_discount_hash_valid, is_event_disputed, is_governor, is_initialized, is_paused,
    is_token_whitelisted, mark_discount_hash_used, remove_payment_from_buyer_index,
    remove_token_from_whitelist, set_admin, set_auction_closed, set_bulk_refund_index,
    set_event_dispute_status, set_event_registry, set_governor, set_highest_bid, set_initialized,
    set_is_paused, set_oracle_address, set_partial_refund_index, set_partial_refund_percentage,
    set_platform_wallet, set_price_switched, set_proposal, set_slippage_bps, set_total_governors,
    set_transfer_fee, set_usdc_token, set_withdrawal_cap, store_payment, store_validation_hash,
    subtract_from_active_escrow_by_token, subtract_from_active_escrow_total,
    subtract_from_total_fees_collected_by_token, update_event_balance, verify_secret,
};
use crate::types::{
    DataKey, HighestBid, ParameterChange, ParameterProposal, Payment, PaymentStatus,
    ProposalStatus, MAX_BPS, TRANSFER_FEE_BPS,
};
use crate::{
    error::TicketPaymentError,
    events::{
        AgoraEvent, AuctionClosedEvent, BidPlacedEvent, BulkRefundProcessedEvent,
        ContractPausedEvent, ContractUpgraded, ContractVerificationFailedEvent,
        DiscountCodeAppliedEvent, DisputeStatusChangedEvent, FeeSettledEvent,
        GlobalPromoAppliedEvent, GovernanceActionExecutedEvent, InitializationEvent,
        PartialRefundProcessedEvent, PaymentProcessedEvent, PaymentStatusChangedEvent,
        PriceSwitchedEvent, ProposalCreatedEvent, ProposalVotedEvent, RevenueClaimedEvent,
        TicketCheckedInEvent, TicketTransferredEvent,
    },
};
use soroban_sdk::{
    contract, contractimpl, token, Address, Bytes, BytesN, Env, String, Symbol, Vec,
};

const MAX_ORACLE_PRICE_AGE_SECS: u64 = 3600;

/// Minimum claimable amount in stroops (0.01 USDC).
/// Balances at or below this threshold are swept in full to avoid dust.
const DUST_THRESHOLD: i128 = 10_000;

// Price Oracle interface
pub mod price_oracle {
    use soroban_sdk::{contractclient, Address, Env};

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct PriceData {
        pub price: i128,
        pub timestamp: u64,
    }

    #[contractclient(name = "OracleClient")]
    pub trait PriceOracleInterface {
        fn lastprice(env: Env, asset: Address) -> Option<PriceData>;
    }
}

// Event Registry interface
pub mod event_registry {
    use soroban_sdk::{contractclient, Address, Env, String};

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum EventStatus {
        Active,
        Inactive,
        Cancelled,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct PaymentInfo {
        pub payment_address: Address,
        pub platform_fee_percent: u32,
        pub custom_fee_bps: Option<u32>,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct EventInventory {
        pub current_supply: i128,
        pub max_supply: i128,
    }

    /// Loyalty profile mirrored from the event_registry contract
    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct GuestProfile {
        pub guest_address: Address,
        pub loyalty_score: u64,
        pub total_tickets_purchased: u32,
        pub total_spent: i128,
        pub last_updated: u64,
    }

    #[contractclient(name = "Client")]
    pub trait EventRegistryInterface {
        fn get_event_payment_info(env: Env, event_id: String) -> PaymentInfo;
        fn get_event(env: Env, event_id: String) -> Option<EventInfo>;
        fn get_organizer_address(env: Env, event_id: String) -> Option<Address>;
        fn increment_inventory(
            env: Env,
            event_id: String,
            tier_id: String,
            user: Address,
            quantity: u32,
        );
        fn decrement_inventory(env: Env, event_id: String, tier_id: String, user: Address);
        fn get_global_promo_bps(env: Env) -> u32;
        fn get_promo_expiry(env: Env) -> u64;
        fn is_scanner_authorized(env: Env, event_id: String, scanner: Address) -> bool;
        fn update_loyalty_score(
            env: Env,
            caller: Address,
            guest: Address,
            tickets_purchased: u32,
            amount_spent: i128,
            loyalty_multiplier: u32,
        );
        fn get_loyalty_discount_bps(env: Env, guest: Address) -> u32;
        fn get_guest_profile(env: Env, guest: Address) -> Option<GuestProfile>;
    }

    pub use crate::types::AuctionConfig;

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct TicketTier {
        pub name: String,
        pub price: i128,
        pub early_bird_price: i128,
        pub early_bird_deadline: u64,
        pub usd_price: i128,
        pub tier_limit: i128,
        pub current_sold: i128,
        pub is_refundable: bool,
        pub auction_config: soroban_sdk::Vec<AuctionConfig>,
        pub loyalty_multiplier: u32,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Milestone {
        pub sales_threshold: i128,
        pub release_percent: u32,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct EventInfo {
        pub event_id: String,
        pub name: String,
        pub organizer_address: Address,
        pub payment_address: Address,
        pub platform_fee_percent: u32,
        pub is_active: bool,
        pub status: EventStatus,
        pub created_at: u64,
        pub metadata_cid: String,
        pub max_supply: i128,
        pub current_supply: i128,
        pub milestone_plan: Option<soroban_sdk::Vec<Milestone>>,
        pub tiers: soroban_sdk::Map<String, TicketTier>,
        pub refund_deadline: u64,
        pub restocking_fee: i128,
        pub resale_cap_bps: Option<u32>,
        pub min_sales_target: i128,
        pub target_deadline: u64,
        pub goal_met: bool,
        pub custom_fee_bps: Option<u32>,
        pub banner_cid: Option<String>,
        pub tags: Option<soroban_sdk::Vec<String>>,
        pub start_time: u64,
        pub end_time: u64,
    }
}

fn require_admin(env: &Env) -> Result<Address, TicketPaymentError> {
    let admin = get_admin(env).ok_or(TicketPaymentError::NotInitialized)?;
    admin.require_auth();
    Ok(admin)
}

#[contract]
pub struct TicketPaymentContract;

#[contractimpl]
#[allow(deprecated)]
impl TicketPaymentContract {
    /// Initializes the contract with necessary configurations.
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc_token: Address,
        platform_wallet: Address,
        event_registry: Address,
    ) -> Result<(), TicketPaymentError> {
        if is_initialized(&env) {
            return Err(TicketPaymentError::AlreadyInitialized);
        }

        validate_address(&env, &admin)?;
        validate_address(&env, &usdc_token)?;
        validate_address(&env, &platform_wallet)?;
        validate_address(&env, &event_registry)?;

        set_admin(&env, &admin);
        set_governor(&env, &admin, true);
        set_total_governors(&env, 1);
        set_usdc_token(&env, usdc_token.clone());
        set_platform_wallet(&env, platform_wallet.clone());
        set_event_registry(&env, event_registry.clone());
        set_initialized(&env, true);

        // Whitelist USDC by default
        add_token_to_whitelist(&env, &usdc_token);

        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::ContractInitialized,),
            InitializationEvent {
                usdc_token,
                platform_wallet,
                event_registry,
            },
        );

        Ok(())
    }

    /// Pauses or resumes the contract. Only callable by the multi-sig admin.
    /// Upgrade and emergency-withdrawal remain available while the contract is paused.
    pub fn set_pause(env: Env, paused: bool) -> Result<(), TicketPaymentError> {
        require_admin(&env)?;
        set_is_paused(&env, paused);
        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::ContractPaused,),
            ContractPausedEvent {
                paused,
                timestamp: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    /// Returns the current paused state of the contract.
    pub fn get_is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    /// Returns true if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    /// Sets or clears a dispute for an event.
    pub fn set_event_dispute(
        env: Env,
        event_id: String,
        disputed: bool,
    ) -> Result<(), TicketPaymentError> {
        require_admin(&env)?;

        set_event_dispute_status(&env, event_id.clone(), disputed);

        env.events().publish(
            (AgoraEvent::DisputeStatusChanged,),
            DisputeStatusChangedEvent {
                event_id,
                is_disputed: disputed,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Returns if an event is currently disputed.
    pub fn is_event_disputed(env: Env, event_id: String) -> bool {
        is_event_disputed(&env, event_id)
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        require_admin(&env).expect("Admin not set");

        let old_wasm_hash = match env.current_contract_address().executable() {
            Some(soroban_sdk::Executable::Wasm(hash)) => hash,
            _ => panic!("Current contract is not a Wasm contract"),
        };

        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());

        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::ContractUpgraded,),
            ContractUpgraded {
                old_wasm_hash,
                new_wasm_hash,
            },
        );

        // Post-upgrade state verification
        let mut missing_keys = soroban_sdk::Vec::new(&env);
        if !env.storage().persistent().has(&DataKey::Admin) {
            missing_keys.push_back(String::from_str(&env, "Admin"));
        }
        if !env.storage().persistent().has(&DataKey::UsdcToken) {
            missing_keys.push_back(String::from_str(&env, "UsdcToken"));
        }
        if !env.storage().persistent().has(&DataKey::PlatformWallet) {
            missing_keys.push_back(String::from_str(&env, "PlatformWallet"));
        }
        if !env.storage().persistent().has(&DataKey::EventRegistry) {
            missing_keys.push_back(String::from_str(&env, "EventRegistry"));
        }

        if !missing_keys.is_empty() {
            for key in missing_keys.iter() {
                env.events().publish(
                    (Symbol::new(&env, "ContractVerificationFailed"),),
                    ContractVerificationFailedEvent {
                        missing_key: key,
                        timestamp: env.ledger().timestamp(),
                    },
                );
            }
        }
    }

    /// Proposes a parameter change for the platform. Only callable by a governor.
    pub fn propose_parameter_change(
        env: Env,
        proposer: Address,
        change: ParameterChange,
    ) -> Result<u64, TicketPaymentError> {
        proposer.require_auth();
        if !is_governor(&env, &proposer) {
            return Err(TicketPaymentError::NotGovernor);
        }

        let proposal_id = increment_proposal_count(&env);
        let proposal = ParameterProposal {
            id: proposal_id,
            proposer: proposer.clone(),
            change: change.clone(),
            status: ProposalStatus::Pending,
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + 604800, // 7 days
            vote_count: 1,                                 // proposer automatically votes
            voters: soroban_sdk::vec![&env, proposer.clone()],
        };

        set_proposal(&env, &proposal);

        env.events().publish(
            (AgoraEvent::ProposalCreated,),
            ProposalCreatedEvent {
                proposal_id,
                proposer,
                change,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(proposal_id)
    }

    /// Votes on an active proposal. Only callable by a governor.
    pub fn vote_on_proposal(
        env: Env,
        voter: Address,
        proposal_id: u64,
    ) -> Result<(), TicketPaymentError> {
        voter.require_auth();
        if !is_governor(&env, &voter) {
            return Err(TicketPaymentError::NotGovernor);
        }

        let mut proposal =
            get_proposal(&env, proposal_id).ok_or(TicketPaymentError::InvalidProposal)?;

        if proposal.status != ProposalStatus::Pending {
            return Err(TicketPaymentError::ProposalNotActive);
        }

        if env.ledger().timestamp() > proposal.expires_at {
            return Err(TicketPaymentError::ProposalExpired);
        }

        if proposal.voters.contains(&voter) {
            return Err(TicketPaymentError::AlreadyVoted);
        }

        proposal.voters.push_back(voter.clone());
        proposal.vote_count += 1;
        set_proposal(&env, &proposal);

        env.events().publish(
            (AgoraEvent::ProposalVoted,),
            ProposalVotedEvent {
                proposal_id,
                voter,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Executes a proposal if it has met the voting threshold and time lock.
    pub fn execute_proposal(
        env: Env,
        executor: Address,
        proposal_id: u64,
    ) -> Result<(), TicketPaymentError> {
        executor.require_auth();
        let mut proposal =
            get_proposal(&env, proposal_id).ok_or(TicketPaymentError::InvalidProposal)?;

        if proposal.status != ProposalStatus::Pending {
            return Err(TicketPaymentError::ProposalNotActive);
        }

        let current_time = env.ledger().timestamp();

        if current_time > proposal.expires_at {
            return Err(TicketPaymentError::ProposalExpired);
        }

        // 48 hours = 48 * 60 * 60 = 172800 seconds
        if current_time < proposal.created_at + 172800 {
            return Err(TicketPaymentError::VotingPeriodNotMet);
        }

        let total_governors = get_total_governors(&env);
        // Simple majority: > 50%
        let threshold = (total_governors / 2) + 1;
        if proposal.vote_count < threshold {
            return Err(TicketPaymentError::InsufficientVotes);
        }

        // Execute the change
        match &proposal.change {
            ParameterChange::AddGovernor(new_governor) => {
                if !is_governor(&env, new_governor) {
                    set_governor(&env, new_governor, true);
                    set_total_governors(&env, total_governors + 1);
                }
            }
            ParameterChange::RemoveGovernor(old_governor) => {
                if total_governors <= 1 {
                    return Err(TicketPaymentError::CannotRemoveLastGovernor);
                }
                if is_governor(&env, old_governor) {
                    set_governor(&env, old_governor, false);
                    set_total_governors(&env, total_governors - 1);
                }
            }
            ParameterChange::AddTokenToWhitelist(token) => {
                add_token_to_whitelist(&env, token);
            }
            ParameterChange::RemoveTokenFromWhitelist(token) => {
                remove_token_from_whitelist(&env, token);
            }
            ParameterChange::UpdateWithdrawalCap(token, cap) => {
                set_withdrawal_cap(&env, token.clone(), *cap);
            }
            ParameterChange::UpdateSlippage(bps) => {
                set_slippage_bps(&env, *bps)?;
            }
            ParameterChange::UpdateTransferFee(event_id, fee) => {
                set_transfer_fee(&env, event_id.clone(), *fee);
            }
        }

        proposal.status = ProposalStatus::Executed;
        set_proposal(&env, &proposal);

        env.events().publish(
            (AgoraEvent::GovernanceActionExecuted,),
            GovernanceActionExecutedEvent {
                proposal_id,
                change: proposal.change.clone(),
                timestamp: current_time,
            },
        );

        Ok(())
    }

    pub fn is_token_allowed(env: Env, token: Address) -> bool {
        is_token_whitelisted(&env, &token)
    }

    /// Sets the oracle contract address. Only callable by admin.
    pub fn set_oracle(env: Env, oracle_address: Address) -> Result<(), TicketPaymentError> {
        require_admin(&env)?;
        set_oracle_address(&env, &oracle_address);
        Ok(())
    }

    /// Returns the current oracle price for an asset.
    pub fn get_asset_price(
        env: Env,
        asset: Address,
    ) -> Result<price_oracle::PriceData, TicketPaymentError> {
        fetch_fresh_asset_price(&env, &asset)
    }

    /// Returns the current slippage tolerance in basis points.
    pub fn get_slippage(env: Env) -> u32 {
        get_slippage_bps(&env)
    }

    /// Processes a payment for an event ticket.
    #[allow(clippy::too_many_arguments)]
    pub fn process_payment(
        env: Env,
        payment_id: String,
        event_id: String,
        ticket_tier_id: String,
        buyer_address: Address,
        token_address: Address,
        amount: i128, // price for ONE ticket
        quantity: u32,
        code_preimage: Option<Bytes>,
        referrer: Option<Address>,
        validation_hash: BytesN<32>,
    ) -> Result<String, TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }
        buyer_address.require_auth();

        if let Some(ref ref_addr) = referrer {
            if ref_addr == &buyer_address {
                return Err(TicketPaymentError::SelfReferralNotAllowed);
            }
        }

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        if quantity == 0 {
            panic!("Quantity must be positive");
        }

        if !is_token_whitelisted(&env, &token_address) {
            return Err(TicketPaymentError::TokenNotWhitelisted);
        }

        let total_amount = amount
            .checked_mul(quantity as i128)
            .ok_or(TicketPaymentError::ArithmeticError)?;

        // Apply platform-wide global promo if active (self-expiring via timestamp check)
        let event_registry_addr_promo = get_event_registry(&env);
        let registry_client_promo = event_registry::Client::new(&env, &event_registry_addr_promo);
        let global_promo_bps = registry_client_promo.get_global_promo_bps();
        let promo_expiry = registry_client_promo.get_promo_expiry();
        let current_ts = env.ledger().timestamp();

        let (after_promo, promo_applied_bps) = if global_promo_bps > 0 && current_ts < promo_expiry
        {
            let discounted = total_amount
                .checked_mul((MAX_BPS - global_promo_bps) as i128)
                .and_then(|v| v.checked_div(MAX_BPS as i128))
                .ok_or(TicketPaymentError::ArithmeticError)?;
            (discounted, global_promo_bps)
        } else {
            (total_amount, 0u32)
        };

        // Optionally apply a discount code (10% off) on top of the promo price
        let (effective_total, discount_code_hash) = if let Some(preimage) = code_preimage {
            let hash: soroban_sdk::BytesN<32> = env.crypto().sha256(&preimage).into();
            if !is_discount_hash_valid(&env, &hash) {
                return Err(TicketPaymentError::InvalidDiscountCode);
            }
            if is_discount_hash_used(&env, &hash) {
                return Err(TicketPaymentError::DiscountCodeUsed);
            }
            // 10% discount
            let discounted = after_promo
                .checked_mul(90)
                .and_then(|v| v.checked_div(100))
                .ok_or(TicketPaymentError::ArithmeticError)?;
            (discounted, Some(hash))
        } else {
            (after_promo, None)
        };
        // 1. Query Event Registry for event info and check inventory
        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            Ok(Ok(None)) => return Err(TicketPaymentError::EventNotFound),
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        if !event_info.is_active
            || matches!(event_info.status, event_registry::EventStatus::Cancelled)
        {
            return Err(TicketPaymentError::EventInactive);
        }

        // Block sales if the event has been locally cancelled for refunds
        if crate::storage::is_event_cancelled_for_refund(&env, &event_id) {
            return Err(TicketPaymentError::EventCancelled);
        }

        let tier = event_info
            .tiers
            .get(ticket_tier_id.clone())
            .ok_or(TicketPaymentError::TierNotFound)?;

        let current_time = env.ledger().timestamp();

        if tier.usd_price > 0 {
            // ── Oracle-based USD pricing ──────────────────────────────────
            let price_data = fetch_fresh_asset_price(&env, &token_address)?;

            // expected = usd_price * oracle_price / 1_0000000
            let expected = tier
                .usd_price
                .checked_mul(price_data.price)
                .and_then(|v| v.checked_div(1_0000000))
                .ok_or(TicketPaymentError::ArithmeticError)?;

            let bps = get_slippage_bps(&env) as i128;
            let min_amount = expected
                .checked_mul(MAX_BPS as i128 - bps)
                .and_then(|v| v.checked_div(MAX_BPS as i128))
                .ok_or(TicketPaymentError::ArithmeticError)?;
            let max_amount = expected
                .checked_mul(MAX_BPS as i128 + bps)
                .and_then(|v| v.checked_div(MAX_BPS as i128))
                .ok_or(TicketPaymentError::ArithmeticError)?;

            if amount < min_amount || amount > max_amount {
                return Err(TicketPaymentError::PriceOutsideSlippage);
            }
        } else {
            // ── Exact token-price matching (existing behaviour) ───────────
            let mut active_price = tier.price;

            if tier.early_bird_deadline > 0 && current_time <= tier.early_bird_deadline {
                active_price = tier.early_bird_price;
            }

            if amount != active_price {
                return Err(TicketPaymentError::InvalidPrice);
            }
        }

        // Check if we just transitioned from early bird to standard
        if tier.early_bird_deadline > 0
            && current_time > tier.early_bird_deadline
            && !has_price_switched(&env, event_id.clone(), ticket_tier_id.clone())
        {
            set_price_switched(&env, event_id.clone(), ticket_tier_id.clone());
            #[allow(deprecated)]
            env.events().publish(
                (AgoraEvent::PriceSwitched,),
                PriceSwitchedEvent {
                    event_id: event_id.clone(),
                    tier_id: ticket_tier_id.clone(),
                    new_price: tier.price,
                    timestamp: current_time,
                },
            );
        }

        // 2. Calculate platform fee
        let fee_bps = event_info
            .custom_fee_bps
            .unwrap_or(event_info.platform_fee_percent);

        let mut total_platform_fee = effective_total
            .checked_mul(fee_bps as i128)
            .and_then(|v| v.checked_div(MAX_BPS as i128))
            .ok_or(TicketPaymentError::ArithmeticError)?;

        // Apply loyalty discount: reduce the platform fee for guests with high scores.
        // Uses try_ variant so that contracts without loyalty support are unaffected.
        let loyalty_discount_bps: u32 = registry_client_promo
            .try_get_loyalty_discount_bps(&buyer_address)
            .ok()
            .and_then(|r| r.ok())
            .unwrap_or(0);

        let loyalty_discount_amount = if loyalty_discount_bps > 0 {
            core::cmp::min(
                total_platform_fee
                    .checked_mul(loyalty_discount_bps as i128)
                    .and_then(|v| v.checked_div(MAX_BPS as i128))
                    .ok_or(TicketPaymentError::ArithmeticError)?,
                total_platform_fee,
            )
        } else {
            0
        };
        total_platform_fee = total_platform_fee
            .checked_sub(loyalty_discount_amount)
            .ok_or(TicketPaymentError::ArithmeticError)?;

        // Adjust effective_total to reflect the loyalty discount (guest pays less)
        let effective_total = effective_total
            .checked_sub(loyalty_discount_amount)
            .ok_or(TicketPaymentError::ArithmeticError)?;

        let total_organizer_amount = effective_total
            .checked_sub(total_platform_fee)
            .ok_or(TicketPaymentError::ArithmeticError)?;

        let referral_reward = if referrer.is_some() {
            let reward = total_platform_fee
                .checked_mul(20)
                .and_then(|v| v.checked_div(100))
                .ok_or(TicketPaymentError::ArithmeticError)?; // 20%
                                                              // Cap: referral reward must never exceed the remaining platform fee.
            let reward = core::cmp::min(reward, total_platform_fee);
            total_platform_fee = total_platform_fee
                .checked_sub(reward)
                .ok_or(TicketPaymentError::ArithmeticError)?;
            reward
        } else {
            0
        };

        // 3. Transfer tokens to contract (escrow)
        let token_client = token::Client::new(&env, &token_address);
        let contract_address = env.current_contract_address();

        // Verify allowance
        let allowance = token_client.allowance(&buyer_address, &contract_address);
        if allowance < effective_total {
            return Err(TicketPaymentError::InsufficientAllowance);
        }

        // Get balance before transfer
        let balance_before = token_client.balance(&contract_address);

        // Transfer full amount to contract
        token_client.transfer_from(
            &contract_address,
            &buyer_address,
            &contract_address,
            &effective_total,
        );

        // Verify balance after transfer
        let balance_after = token_client.balance(&contract_address);
        if balance_after
            .checked_sub(balance_before)
            .ok_or(TicketPaymentError::ArithmeticError)?
            != effective_total
        {
            return Err(TicketPaymentError::TransferVerificationFailed);
        }

        // Transfer referral reward if applicable
        if let Some(ref ref_addr) = referrer {
            if referral_reward > 0 {
                token_client.transfer(&contract_address, ref_addr, &referral_reward);
            }
        }

        // 4. Update escrow balances
        update_event_balance(
            &env,
            event_id.clone(),
            total_organizer_amount,
            total_platform_fee,
        );
        add_to_total_volume_processed(&env, total_amount);
        add_to_total_fees_collected_by_token(&env, token_address.clone(), total_platform_fee);
        add_to_active_escrow_total(&env, total_amount);
        add_to_active_escrow_by_token(&env, token_address.clone(), total_amount);

        // 5. Mark the discount code as used (after funds are safely transferred)
        if let Some(hash) = discount_code_hash.clone() {
            mark_discount_hash_used(&env, hash);
        }

        // 6. Increment inventory after successful payment
        registry_client.increment_inventory(
            &event_id,
            &ticket_tier_id,
            &buyer_address,
            &quantity,
        );

        // 7. Create payment records for each individual ticket
        let quantity_i128 = quantity as i128;
        let platform_fee_per_ticket = total_platform_fee
            .checked_div(quantity_i128)
            .ok_or(TicketPaymentError::ArithmeticError)?;
        let organizer_amount_per_ticket = total_organizer_amount
            .checked_div(quantity_i128)
            .ok_or(TicketPaymentError::ArithmeticError)?;
        let created_at = env.ledger().timestamp();
        let empty_tx_hash = String::from_str(&env, "");

        for i in 0..quantity {
            // Re-initialize the sub_payment_id with a unique ID for each ticket in a batch.
            // Since concatenation is complex in Soroban no_std, we use a match for common indices.
            let sub_payment_id = if quantity == 1 {
                payment_id.clone()
            } else {
                match i {
                    0 => String::from_str(&env, "p-0"),
                    1 => String::from_str(&env, "p-1"),
                    2 => String::from_str(&env, "p-2"),
                    3 => String::from_str(&env, "p-3"),
                    4 => String::from_str(&env, "p-4"),
                    _ => String::from_str(&env, "p-many"),
                }
            };

            let payment = Payment {
                payment_id: sub_payment_id.clone(),
                event_id: event_id.clone(),
                buyer_address: buyer_address.clone(),
                ticket_tier_id: ticket_tier_id.clone(),
                amount,
                platform_fee: platform_fee_per_ticket,
                organizer_amount: organizer_amount_per_ticket,
                status: PaymentStatus::Pending,
                transaction_hash: empty_tx_hash.clone(),
                created_at,
                confirmed_at: None,
                refunded_amount: 0,
                is_soulbound: false,
                last_checked_in_at: 0,
            };

            store_payment(&env, payment);
            store_validation_hash(&env, &sub_payment_id, &validation_hash);
        }

        // 8. Emit payment event
        env.events().publish(
            (AgoraEvent::PaymentProcessed,),
            PaymentProcessedEvent {
                payment_id: payment_id.clone(),
                event_id: event_id.clone(),
                buyer_address: buyer_address.clone(),
                amount: effective_total,
                platform_fee: total_platform_fee,
                timestamp: env.ledger().timestamp(),
            },
        );

        // 8a. Award loyalty points to buyer (best-effort; ignore failures)
        match registry_client_promo.try_update_loyalty_score(
            &env.current_contract_address(),
            &buyer_address,
            &quantity,
            &effective_total,
            &tier.loyalty_multiplier,
        ) {
            Ok(_) | Err(_) => {}
        }

        // 9. Emit discount applied event if a code was used
        if let Some(hash) = discount_code_hash {
            let discount_amount = total_amount.checked_sub(effective_total).unwrap_or(0);
            env.events().publish(
                (AgoraEvent::DiscountCodeApplied,),
                DiscountCodeAppliedEvent {
                    payment_id: payment_id.clone(),
                    event_id: event_id.clone(),
                    code_hash: hash,
                    discount_amount,
                    timestamp: env.ledger().timestamp(),
                },
            );
        }

        // 10. Emit global promo applied event if promo was active
        if promo_applied_bps > 0 {
            let promo_discount_amount = total_amount.checked_sub(after_promo).unwrap_or(0);
            env.events().publish(
                (AgoraEvent::GlobalPromoApplied,),
                GlobalPromoAppliedEvent {
                    payment_id: payment_id.clone(),
                    event_id: event_id.clone(),
                    promo_bps: promo_applied_bps,
                    discount_amount: promo_discount_amount,
                    timestamp: env.ledger().timestamp(),
                },
            );
        }

        Ok(payment_id)
    }

    /// Confirms a payment after backend verification.
    pub fn confirm_payment(env: Env, payment_id: String, transaction_hash: String) {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        require_admin(&env).expect("Admin not set");
        if let Some(mut payment) = get_payment(&env, payment_id.clone()) {
            let old_status = payment.status.clone();
            payment.status = PaymentStatus::Confirmed;
            payment.confirmed_at = Some(env.ledger().timestamp());
            payment.transaction_hash = transaction_hash.clone();

            // Update payment in storage
            let key = DataKey::Payment(payment_id.clone());
            env.storage().persistent().set(&key, &payment);

            // Update status index
            if old_status != PaymentStatus::Confirmed {
                crate::storage::update_payment_status_index(
                    &env,
                    payment.event_id.clone(),
                    old_status.clone(),
                    PaymentStatus::Confirmed,
                    payment_id.clone(),
                );
            }
        }

        // Emit confirmation event
        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::PaymentStatusChanged,),
            PaymentStatusChangedEvent {
                payment_id: payment_id.clone(),
                old_status: PaymentStatus::Pending,
                new_status: PaymentStatus::Confirmed,
                transaction_hash: transaction_hash.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );
    }

    pub fn request_guest_refund(env: Env, payment_id: String) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        Self::internal_refund(env, payment_id)
    }

    /// Requests a refund for a ticket identified by a numeric ticket_id.
    ///
    /// Fails with `RefundDeadlinePassed` if the current timestamp exceeds the event's
    /// `refund_deadline`. Also fails if the ticket has already been used (checked-in)
    /// or refunded. On success, transfers `amount - restocking_fee` back to the buyer
    /// and marks the ticket as `Refunded`.
    pub fn request_refund(env: Env, ticket_id: u64) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        let payment_id = get_ticket_payment_id(&env, ticket_id)
            .ok_or(TicketPaymentError::PaymentNotFound)?;

        let payment =
            get_payment(&env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

        // Reject if ticket is already used (checked-in) or already refunded
        if payment.status == PaymentStatus::CheckedIn {
            return Err(TicketPaymentError::TicketAlreadyUsed);
        }
        if payment.status == PaymentStatus::Refunded {
            return Err(TicketPaymentError::InvalidPaymentStatus);
        }

        // Verify refund deadline
        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);
        let event_info = match registry_client.try_get_event(&payment.event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        let is_cancelled = matches!(event_info.status, event_registry::EventStatus::Cancelled);
        if !is_cancelled
            && event_info.refund_deadline > 0
            && env.ledger().timestamp() > event_info.refund_deadline
        {
            return Err(TicketPaymentError::RefundDeadlinePassed);
        }

        Self::internal_refund(env, payment_id)
    }

    /// Triggers a refund as an administrator, regardless of dispute status.
    pub fn admin_refund(env: Env, payment_id: String) -> Result<(), TicketPaymentError> {
        require_admin(&env)?;

        Self::internal_refund(env, payment_id)
    }

    /// Public wrapper for automatic refunds, specifically for cancelled events.
    pub fn claim_automatic_refund(env: Env, payment_id: String) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        let payment =
            get_payment(&env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&payment.event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        // Ensure the event is cancelled for automatic refund OR goal failed after deadline
        let current_ts = env.ledger().timestamp();
        let goal_failed = !event_info.goal_met
            && event_info.min_sales_target > 0
            && current_ts > event_info.target_deadline;

        if !matches!(event_info.status, event_registry::EventStatus::Cancelled) && !goal_failed {
            return Err(TicketPaymentError::InvalidPaymentStatus);
        }

        Self::internal_refund(env, payment_id)
    }

    fn internal_refund(env: Env, payment_id: String) -> Result<(), TicketPaymentError> {
        let mut payment =
            get_payment(&env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

        payment.buyer_address.require_auth();

        if payment.status == PaymentStatus::Refunded || payment.status == PaymentStatus::Failed {
            return Err(TicketPaymentError::InvalidPaymentStatus);
        }

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&payment.event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        let tier = event_info
            .tiers
            .get(payment.ticket_tier_id.clone())
            .ok_or(TicketPaymentError::TierNotFound)?;

        let is_cancelled = matches!(event_info.status, event_registry::EventStatus::Cancelled);
        let current_ts = env.ledger().timestamp();
        let goal_failed = !event_info.goal_met
            && event_info.min_sales_target > 0
            && current_ts > event_info.target_deadline;

        // Check if refundable or if EVENT IS CANCELLED or GOAL FAILED
        if !tier.is_refundable && !is_cancelled && !goal_failed && event_info.is_active {
            return Err(TicketPaymentError::TicketNotRefundable);
        }

        // Validate against refund deadline if event is active and not cancelled
        if !is_cancelled
            && event_info.is_active
            && event_info.refund_deadline > 0
            && env.ledger().timestamp() > event_info.refund_deadline
        {
            return Err(TicketPaymentError::RefundDeadlinePassed);
        }

        // Deduct restocking fee if specified (capped at payment amount)
        // Bypass restocking fee if the event is cancelled or goal failed.
        let effective_restocking_fee = if is_cancelled || goal_failed {
            0
        } else if event_info.restocking_fee > payment.amount {
            payment.amount
        } else if event_info.restocking_fee > 0 {
            event_info.restocking_fee
        } else {
            0
        };

        let refund_amount = payment
            .amount
            .checked_sub(effective_restocking_fee)
            .ok_or(TicketPaymentError::ArithmeticError)?;

        // Return ticket to inventory (increments available inventory)
        registry_client.decrement_inventory(
            &payment.event_id,
            &payment.ticket_tier_id,
            &payment.buyer_address,
        );

        let old_status = payment.status.clone();
        payment.status = PaymentStatus::Refunded;
        payment.confirmed_at = Some(env.ledger().timestamp());

        // Update payment in storage
        let key = DataKey::Payment(payment_id.clone());
        env.storage().persistent().set(&key, &payment);

        // Update status index
        if old_status != PaymentStatus::Refunded {
            crate::storage::update_payment_status_index(
                &env,
                payment.event_id.clone(),
                old_status.clone(),
                PaymentStatus::Refunded,
                payment_id.clone(),
            );
        }

        // Process token transfer
        if refund_amount > 0 {
            let token_address = crate::storage::get_usdc_token(&env);
            token::Client::new(&env, &token_address).transfer(
                &env.current_contract_address(),
                &payment.buyer_address,
                &refund_amount,
            );
        }

        // Guest receives payment.amount - effective_restocking_fee
        // Organizer keeps effective_restocking_fee (adjust from original organizer_amount)
        // Platform fee is refunded (removed from escrow)
        let org_adjustment = payment
            .organizer_amount
            .checked_sub(effective_restocking_fee)
            .ok_or(TicketPaymentError::ArithmeticError)?;
        let platform_adjustment = payment.platform_fee;

        crate::storage::update_event_balance(
            &env,
            payment.event_id.clone(),
            -org_adjustment,
            -platform_adjustment,
        );

        subtract_from_active_escrow_total(&env, refund_amount);
        subtract_from_active_escrow_by_token(
            &env,
            crate::storage::get_usdc_token(&env),
            refund_amount,
        );

        // Clear escrow record if both amounts are now zero (fully refunded event)
        let updated_balance = get_event_balance(&env, payment.event_id.clone());
        if updated_balance.organizer_amount == 0 && updated_balance.platform_fee == 0 {
            // Keep the record but ensure it's clean
            update_event_balance(&env, payment.event_id.clone(), 0, 0);
        }

        // Emit confirmation event
        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::PaymentStatusChanged,),
            PaymentStatusChangedEvent {
                payment_id: payment_id.clone(),
                old_status,
                new_status: PaymentStatus::Refunded,
                transaction_hash: String::from_str(&env, "refund"),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }
    pub fn get_payment_status(env: Env, payment_id: String) -> Option<Payment> {
        get_payment(&env, payment_id)
    }

    // pub fn check_in(
    //     env: Env,
    //     payment_id: String,
    //     scanner: Address,
    //     // Optionally, a series_id and pass_holder for series pass check-in
    //     series_id: Option<String>,
    //     pass_holder: Option<Address>,
    // ) -> Result<(), TicketPaymentError> {
    //     if !is_initialized(&env) {
    //         panic!("Contract not initialized");
    //     }
    //     if is_paused(&env) {
    //         return Err(TicketPaymentError::ContractPaused);
    //     }

    //     // If series_id and pass_holder are provided, check for a valid series pass
    //     if let (Some(series_id), Some(holder)) = (series_id.clone(), pass_holder.clone()) {
    //         // Call EventRegistry to verify pass
    //         let event_registry_addr = get_event_registry(&env);
    //         let registry_client = event_registry::Client::new(&env, &event_registry_addr);
    //         // Check if event is part of the series
    //         let series_opt: Option<_> = registry_client.get_series(&series_id);
    //         if let Some(series) = series_opt {
    //             let found = series.event_ids.iter().any(|eid| eid == payment_id);
    //             if !found {
    //                 return Err(TicketPaymentError::EventNotFound);
    //             }
    //             // Get the pass for the holder
    //             let pass_opt = registry_client.get_holder_series_pass(&holder, &series_id);
    //             if let Some(mut pass) = pass_opt {
    //                 // Check expiry
    //                 if pass.expires_at > 0 && env.ledger().timestamp() > pass.expires_at {
    //                     return Err(TicketPaymentError::InvalidPaymentStatus); // Use a better error if available
    //                 }
    //                 // Check usage limit
    //                 if pass.usage_count >= pass.usage_limit {
    //                     return Err(TicketPaymentError::TicketAlreadyUsed);
    //                 }
    //                 // Increment usage
    //                 let _ = registry_client.increment_series_pass_usage(&pass.pass_id);
    //                 // Emit event for series pass check-in (optional)
    //                 // (You may want to add a new event type for this)
    //                 return Ok(());
    //             } else {
    //                 return Err(TicketPaymentError::PaymentNotFound);
    //             }
    //         } else {
    //             return Err(TicketPaymentError::EventNotFound);
    //         }
    //     }

    //     // Fallback: normal ticket check-in
    //     let mut payment =
    //         get_payment(&env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

    //     // Must authenticate the scanner wallet calling this entry point
    //     scanner.require_auth();

    //     if payment.status == PaymentStatus::CheckedIn {
    //         return Err(TicketPaymentError::TicketAlreadyUsed);
    //     }

    //     // Verify scanner authorization
    //     let event_registry_addr = get_event_registry(&env);
    //     let registry_client = event_registry::Client::new(&env, &event_registry_addr);
    //     let is_auth = registry_client.is_scanner_authorized(&payment.event_id, &scanner);
    //     if !is_auth {
    //         return Err(TicketPaymentError::UnauthorizedScanner);
    //     }

    //     // Update status and store arrival timestamp
    //     payment.status = PaymentStatus::CheckedIn;
    //     payment.confirmed_at = Some(env.ledger().timestamp());

    //     store_payment(&env, payment.clone());

    //     #[allow(deprecated)]
    //     env.events().publish(
    //         (AgoraEvent::TicketCheckedIn,),
    //         crate::events::TicketCheckedInEvent {
    //             payment_id,
    //             event_id: payment.event_id,
    //             scanner,
    //             timestamp: env.ledger().timestamp(),
    //         },
    //     );

    //     Ok(())
    // }

    /// Verifies scanner authorization and marks a ticket as CheckedIn.
    pub fn check_in(
        env: Env,
        payment_id: String,
        scanner: Address,
        _series_id: Option<String>,
        _pass_holder: Option<Address>,
        raw_secret: Bytes,
    ) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        // ── Temporarily disabled ── series pass logic ────────────────────────
        /*
        if let (Some(series_id), Some(holder)) = (series_id, pass_holder) {
            let registry_client = event_registry::Client::new(&env, &get_event_registry(&env));
            let series_opt = registry_client.get_series(&series_id);           // ← does not exist
            // ...
        }
        */

        // Normal single-ticket check-in
        let mut payment =
            get_payment(&env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

        scanner.require_auth();

        if payment.status == PaymentStatus::CheckedIn {
            return Err(TicketPaymentError::TicketAlreadyUsed);
        }

        // Verify the raw secret matches the stored validation hash
        if !verify_secret(&env, &payment_id, &raw_secret) {
            return Err(TicketPaymentError::InvalidSecret);
        }

        let registry_client = event_registry::Client::new(&env, &get_event_registry(&env));

        // Check if the event has ended (prevent check-ins after end_time)
        let event_info = registry_client
            .try_get_event(&payment.event_id)
            .ok()
            .and_then(|r| r.ok())
            .flatten()
            .ok_or(TicketPaymentError::EventNotFound)?;

        // Allow organizer OR an authorized scanner.
        let is_organizer = scanner == event_info.organizer_address;
        let is_scanner = registry_client.is_scanner_authorized(&payment.event_id, &scanner);
        if !is_organizer && !is_scanner {
            return Err(TicketPaymentError::UnauthorizedScanner);
        }

        let current_time = env.ledger().timestamp();
        if event_info.end_time > 0 && current_time > event_info.end_time {
            return Err(TicketPaymentError::EventEnded);
        }

        payment.status = PaymentStatus::CheckedIn;
        payment.last_checked_in_at = current_time;
        store_payment(&env, payment.clone());

        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::TicketCheckedIn,),
            TicketCheckedInEvent {
                payment_id,
                event_id: payment.event_id,
                attendee: payment.buyer_address,
                scanner,
                timestamp: current_time,
            },
        );

        Ok(())
    }
    /// Returns the escrowed balance for an event.
    pub fn get_event_escrow_balance(env: Env, event_id: String) -> crate::types::EventBalance {
        get_event_balance(&env, event_id)
    }

    /// Withdraw organizer funds from escrow.
    pub fn withdraw_organizer_funds(
        env: Env,
        event_id: String,
        token_address: Address,
    ) -> Result<i128, TicketPaymentError> {
        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);
        let event_info = registry_client
            .try_get_event(&event_id)
            .ok()
            .and_then(|r| r.ok())
            .flatten()
            .ok_or(TicketPaymentError::EventNotFound)?;

        event_info.organizer_address.require_auth();

        let balance = get_event_balance(&env, event_id.clone());
        // Block all claim_revenue attempts for an event while a dispute is active.
        if is_event_disputed(&env, event_id.clone()) {
            return Err(TicketPaymentError::EventDisputed);
        }

        // Block any further organizer payouts once an event is in the Cancelled state.
        if matches!(event_info.status, event_registry::EventStatus::Cancelled) {
            return Err(TicketPaymentError::EventCancelled);
        }

        // Check if goal was met if a target was set
        if event_info.min_sales_target > 0 && !event_info.goal_met {
            return Err(TicketPaymentError::GoalNotMet);
        }

        let total_revenue = balance
            .organizer_amount
            .checked_add(balance.total_withdrawn)
            .ok_or(TicketPaymentError::ArithmeticError)?;
        if total_revenue == 0 {
            return Ok(0);
        }

        let mut release_percent = MAX_BPS;
        if let Some(milestones) = event_info.milestone_plan {
            let mut highest_met = 0u32;
            for milestone in milestones.iter() {
                if event_info.current_supply >= milestone.sales_threshold
                    && milestone.release_percent > highest_met
                {
                    highest_met = milestone.release_percent;
                }
            }
            if !milestones.is_empty() {
                release_percent = highest_met;
            }
        }

        let max_allowed = total_revenue
            .checked_mul(release_percent as i128)
            .and_then(|v| v.checked_div(MAX_BPS as i128))
            .ok_or(TicketPaymentError::ArithmeticError)?;
        let mut available_to_withdraw = max_allowed
            .checked_sub(balance.total_withdrawn)
            .ok_or(TicketPaymentError::ArithmeticError)?;

        if available_to_withdraw <= 0 {
            return Ok(0);
        }

        if available_to_withdraw > balance.organizer_amount {
            available_to_withdraw = balance.organizer_amount;
        }

        token::Client::new(&env, &token_address).transfer(
            &env.current_contract_address(),
            &event_info.organizer_address,
            &available_to_withdraw,
        );

        crate::storage::set_event_balance(
            &env,
            event_id,
            crate::types::EventBalance {
                organizer_amount: balance
                    .organizer_amount
                    .checked_sub(available_to_withdraw)
                    .ok_or(TicketPaymentError::ArithmeticError)?,
                total_withdrawn: balance
                    .total_withdrawn
                    .checked_add(available_to_withdraw)
                    .ok_or(TicketPaymentError::ArithmeticError)?,
                platform_fee: balance.platform_fee,
            },
        );
        subtract_from_active_escrow_total(&env, available_to_withdraw);
        subtract_from_active_escrow_by_token(&env, token_address, available_to_withdraw);

        Ok(available_to_withdraw)
    }

    /// Settles platform fees from an event escrow into the global treasury pool.
    pub fn settle_platform_fees(
        env: Env,
        event_id: String,
        _token_address: Address,
    ) -> Result<i128, TicketPaymentError> {
        require_admin(&env)?;

        let balance = get_event_balance(&env, event_id.clone());
        if balance.platform_fee == 0 {
            return Ok(0);
        }

        // We clarify that these are now "Settled" but they remain in the contract
        // until a bulk withdrawal is made via `withdraw_platform_fees`.
        crate::storage::set_event_balance(
            &env,
            event_id.clone(),
            crate::types::EventBalance {
                organizer_amount: balance.organizer_amount,
                total_withdrawn: balance.total_withdrawn,
                platform_fee: 0,
            },
        );

        // Emit settlement event
        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::FeeSettled,),
            FeeSettledEvent {
                event_id,
                platform_wallet: get_platform_wallet(&env),
                fee_amount: balance.platform_fee,
                fee_bps: 0, // Not applicable here
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(balance.platform_fee)
    }

    /// Withdraw accumulated platform fees from the contract treasury.
    /// Incorporates a daily withdrawal cap and requires admin (multi-sig) authorization.
    pub fn withdraw_platform_fees(
        env: Env,
        amount: i128,
        token_address: Address,
    ) -> Result<(), TicketPaymentError> {
        require_admin(&env)?;

        if amount <= 0 {
            return Err(TicketPaymentError::ArithmeticError);
        }

        // 1. Verify that the amount requested is less than or equal to the recorded total_fees_collected.
        let total_accumulated = get_total_fees_collected_by_token(&env, token_address.clone());
        if amount > total_accumulated {
            return Err(TicketPaymentError::InsufficientFees);
        }

        // 2. Incorporate a 'Withdrawal Cap' per day.
        let cap = get_withdrawal_cap(&env, token_address.clone());
        if cap > 0 {
            let current_day = env.ledger().timestamp() / 86400;
            let already_withdrawn =
                get_daily_withdrawn_amount(&env, token_address.clone(), current_day);
            if already_withdrawn
                .checked_add(amount)
                .ok_or(TicketPaymentError::ArithmeticError)?
                > cap
            {
                return Err(TicketPaymentError::WithdrawalCapExceeded);
            }
            add_to_daily_withdrawn_amount(&env, token_address.clone(), current_day, amount);
        }

        // 3. Process the transfer
        let platform_wallet = get_platform_wallet(&env);
        token::Client::new(&env, &token_address).transfer(
            &env.current_contract_address(),
            &platform_wallet,
            &amount,
        );

        // 4. Update global accounting
        subtract_from_total_fees_collected_by_token(&env, token_address.clone(), amount);
        subtract_from_active_escrow_total(&env, amount);
        subtract_from_active_escrow_by_token(&env, token_address, amount);

        Ok(())
    }

    /// Sets a daily withdrawal cap for a specific token.
    pub fn set_withdrawal_cap(
        env: Env,
        token: Address,
        amount: i128,
    ) -> Result<(), TicketPaymentError> {
        require_admin(&env)?;

        if amount < 0 {
            return Err(TicketPaymentError::ArithmeticError);
        }

        set_withdrawal_cap(&env, token, amount);
        Ok(())
    }

    /// Claim revenue after event completion.
    pub fn claim_revenue(
        env: Env,
        event_id: String,
        token_address: Address,
    ) -> Result<i128, TicketPaymentError> {
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }
        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = registry_client
            .try_get_event(&event_id)
            .ok()
            .and_then(|r| r.ok())
            .flatten()
            .ok_or(TicketPaymentError::EventNotFound)?;

        event_info.organizer_address.require_auth();

        if event_info.is_active
            || !matches!(event_info.status, event_registry::EventStatus::Inactive)
        {
            return Err(TicketPaymentError::EventNotCompleted);
        }

        // Check if goal was met if a target was set
        if event_info.min_sales_target > 0 && !event_info.goal_met {
            return Err(TicketPaymentError::GoalNotMet);
        }

        let balance = get_event_balance(&env, event_id.clone());
        if balance.organizer_amount == 0 && balance.platform_fee == 0 {
            return Err(TicketPaymentError::NoFundsAvailable);
        }

        let platform_wallet = get_platform_wallet(&env);
        let token_client = token::Client::new(&env, &token_address);
        let contract_address = env.current_contract_address();
        let timestamp = env.ledger().timestamp();

        let platform_fee_amount = balance.platform_fee;
        let organizer_amount = balance.organizer_amount;

        // If the organizer's remaining balance is at or below the dust threshold,
        // sweep the full contract balance for this event to avoid leaving tiny amounts.
        let contract_token_balance = token_client.balance(&contract_address);
        let organizer_amount = if organizer_amount > 0
            && organizer_amount <= DUST_THRESHOLD
            && contract_token_balance >= organizer_amount
        {
            contract_token_balance
                .checked_sub(platform_fee_amount)
                .unwrap_or(organizer_amount)
                .max(organizer_amount)
        } else {
            organizer_amount
        };

        // Settlement logic: platform fees stay in the contract but are cleared from EventBalance.
        // They are already tracked in TotalFeesCollected.
        if platform_fee_amount > 0 {
            #[allow(deprecated)]
            env.events().publish(
                (AgoraEvent::FeeSettled,),
                FeeSettledEvent {
                    event_id: event_id.clone(),
                    platform_wallet: platform_wallet.clone(),
                    fee_amount: platform_fee_amount,
                    fee_bps: event_info
                        .custom_fee_bps
                        .unwrap_or(event_info.platform_fee_percent),
                    timestamp,
                },
            );
        }

        // Transfer net revenue to organizer
        if organizer_amount > 0 {
            token_client.transfer(
                &contract_address,
                &event_info.payment_address,
                &organizer_amount,
            );
        }

        // Update balances
        crate::storage::set_event_balance(
            &env,
            event_id.clone(),
            crate::types::EventBalance {
                organizer_amount: 0,
                total_withdrawn: balance.total_withdrawn + organizer_amount,
                platform_fee: 0,
            },
        );

        let total_transferred = organizer_amount;
        if total_transferred > 0 {
            subtract_from_active_escrow_total(&env, total_transferred);
            subtract_from_active_escrow_by_token(&env, token_address, total_transferred);
        }

        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::RevenueClaimed,),
            RevenueClaimedEvent {
                event_id,
                organizer_address: event_info.organizer_address,
                amount: organizer_amount,
                timestamp,
            },
        );

        Ok(organizer_amount)
    }

    /// Returns all payments for a specific buyer.
    pub fn get_buyer_payments(env: Env, buyer_address: Address) -> soroban_sdk::Vec<String> {
        crate::storage::get_buyer_payments(&env, buyer_address)
    }

    /// Returns all payment IDs for an event with a specific status.
    /// This allows querying payments by status (e.g., Pending, Confirmed, Refunded, Failed, CheckedIn).
    ///
    /// # Arguments
    /// * `event_id` - The event ID to query
    /// * `status` - The payment status to filter by
    ///
    /// # Returns
    /// A vector of payment IDs matching the specified status for the event
    pub fn get_payments_by_status(
        env: Env,
        event_id: String,
        status: PaymentStatus,
    ) -> soroban_sdk::Vec<String> {
        crate::storage::get_payments_by_status(&env, event_id, status)
    }

    /// Sets the transfer fee for an event. Only the organizer can call this.
    pub fn set_transfer_fee(
        env: Env,
        event_id: String,
        bps: u32, // Changed from i128 amount to u32 basis points
    ) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        event_info.organizer_address.require_auth();

        if bps > MAX_BPS {
            return Err(TicketPaymentError::InvalidFeePercent);
        }

        // Store the basis points, not the calculated amount
        set_transfer_fee(&env, event_id, bps);
        Ok(())
    }

    /// Transfers a ticket from the current holder to a new owner.
    /// If `sale_price` is provided, it is validated against the event's resale cap.
    pub fn transfer_ticket(
        env: Env,
        payment_id: String,
        to: Address,
        sale_price: Option<i128>,
    ) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        let mut payment =
            get_payment(&env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

        if payment.status != PaymentStatus::Confirmed {
            return Err(TicketPaymentError::InvalidPaymentStatus);
        }

        if payment.is_soulbound {
            return Err(TicketPaymentError::NonTransferable);
        }

        let from = payment.buyer_address.clone();
        from.require_auth();

        // Prevent self-transfer
        if from == to {
            return Err(TicketPaymentError::InvalidAddress);
        }
        // Prevent transfer to the zero address or the contract itself
        validate_recipient(&env, &to)?;

        // Validate resale price against the organizer's cap
        if let Some(price) = sale_price {
            let event_registry_addr = get_event_registry(&env);
            let registry_client = event_registry::Client::new(&env, &event_registry_addr); // Re-initialize client

            let event_info = match registry_client.try_get_event(&payment.event_id) {
                Ok(Ok(Some(info))) => info,
                _ => return Err(TicketPaymentError::EventNotFound), // Should not happen if payment exists
            };

            // Check if resale cap is defined for the event
            if event_info.resale_cap_bps.is_some() {
                if let Some(cap_bps) = event_info.resale_cap_bps {
                    // Look up the original tier face-value price
                    let tier = event_info
                        .tiers
                        .get(payment.ticket_tier_id.clone())
                        .ok_or(TicketPaymentError::TierNotFound)?;
                    // Use the original price from the payment record, not the tier's current price,
                    // as tier prices can change (e.g., early bird expiry).
                    // The payment.amount is the actual price paid for this specific ticket.
                    let original_price = tier.price;

                    // max_price = original_price * (MAX_BPS + cap_bps) / MAX_BPS
                    let max_price = original_price
                        .checked_mul(
                            (MAX_BPS as i128)
                                .checked_add(cap_bps as i128)
                                .unwrap_or(i128::MAX),
                        )
                        .ok_or(TicketPaymentError::ArithmeticError)?
                        / (MAX_BPS as i128);

                    if price > max_price {
                        return Err(TicketPaymentError::ResalePriceExceedsCap);
                    }
                }
            }
        }

        let transfer_fee_bps =
            get_transfer_fee(&env, payment.event_id.clone()).unwrap_or(TRANSFER_FEE_BPS);

        let mut actual_transfer_fee: i128 = 0;

        if transfer_fee_bps > 0 {
            // Calculate the actual transfer fee based on the original ticket amount
            actual_transfer_fee = payment
                .amount
                .checked_mul(transfer_fee_bps as i128)
                .and_then(|v| v.checked_div(MAX_BPS as i128))
                .ok_or(TicketPaymentError::ArithmeticError)?;

            if actual_transfer_fee > 0 {
                let token_address = crate::storage::get_usdc_token(&env);
                let token_client = token::Client::new(&env, &token_address);
                let contract_address = env.current_contract_address();

                // Transfer fee from old owner to contract
                token_client.transfer_from(
                    &contract_address,
                    &from,
                    &contract_address,
                    &actual_transfer_fee,
                );

                // Update escrow balances (fee goes to organizer)
                update_event_balance(&env, payment.event_id.clone(), actual_transfer_fee, 0);
            }
        }
        // Update payment record
        payment.buyer_address = to.clone();
        let key = crate::types::DataKey::Payment(payment_id.clone());
        env.storage().persistent().set(&key, &payment);

        // Update indices
        remove_payment_from_buyer_index(&env, from.clone(), payment_id.clone());
        add_payment_to_buyer_index(&env, to.clone(), payment_id.clone());

        // Emit transfer event
        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::TicketTransferred,),
            TicketTransferredEvent {
                payment_id,
                from,
                to: to.clone(),
                transfer_fee: actual_transfer_fee,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Triggers a bulk refund for a cancelled event. Processes in batches.
    pub fn trigger_bulk_refund(
        env: Env,
        event_id: String,
        batch_size: u32,
    ) -> Result<u32, TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        event_info.organizer_address.require_auth();

        if event_info.is_active
            || !matches!(event_info.status, event_registry::EventStatus::Cancelled)
        {
            return Err(TicketPaymentError::EventNotCompleted);
        }

        let start_index = get_bulk_refund_index(&env, event_id.clone());
        let payment_ids = get_event_payments(&env, event_id.clone());
        let total_payments = payment_ids.len();

        if start_index >= total_payments {
            return Ok(0);
        }

        let end_index = core::cmp::min(start_index + batch_size, total_payments);
        let mut processed_count = 0;
        let mut total_refunded = 0;
        let mut balance = get_event_balance(&env, event_id.clone());

        let token_address = crate::storage::get_usdc_token(&env);
        let token_client = token::Client::new(&env, &token_address);
        let contract_address = env.current_contract_address();

        for i in start_index..end_index {
            let payment_id = payment_ids.get(i).unwrap();
            if let Some(mut payment) = get_payment(&env, payment_id.clone()) {
                if payment.status == PaymentStatus::Confirmed {
                    // Refund full amount to buyer
                    token_client.transfer(
                        &contract_address,
                        &payment.buyer_address,
                        &payment.amount,
                    );

                    // Update payment status
                    payment.status = PaymentStatus::Refunded;
                    payment.confirmed_at = Some(env.ledger().timestamp());
                    store_payment(&env, payment.clone());

                    // Update event balance in-memory; persist once per batch.
                    balance.organizer_amount -= payment.organizer_amount;
                    balance.platform_fee -= payment.platform_fee;

                    total_refunded += payment.amount;
                    processed_count += 1;
                }
            }
        }

        if processed_count > 0 {
            crate::storage::set_event_balance(&env, event_id.clone(), balance);
            subtract_from_active_escrow_total(&env, total_refunded);
            subtract_from_active_escrow_by_token(&env, token_address, total_refunded);
        }

        set_bulk_refund_index(&env, event_id.clone(), end_index);

        // Emit bulk refund event
        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::BulkRefundProcessed,),
            BulkRefundProcessedEvent {
                event_id,
                refund_count: processed_count,
                total_refunded,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(processed_count)
    }

    /// Issues a partial refund to all guests for an event. Processes in batches.
    /// `percentage_bps` is the refund percentage in basis points (e.g., 2000 = 20%).
    pub fn issue_partial_refund(
        env: Env,
        event_id: String,
        percentage_bps: u32,
        batch_size: u32,
    ) -> Result<u32, TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }
        if percentage_bps > MAX_BPS {
            panic!("Percentage cannot exceed 100%");
        }

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        event_info.organizer_address.require_auth();

        let start_index = get_partial_refund_index(&env, event_id.clone());
        let payment_ids = get_event_payments(&env, event_id.clone());
        let total_payments = payment_ids.len();

        if start_index >= total_payments {
            // Check if we were in the middle of a refund and just finished
            let active_pct = get_partial_refund_percentage(&env, event_id.clone());
            if active_pct > 0 {
                set_partial_refund_percentage(&env, event_id.clone(), 0);
                set_partial_refund_index(&env, event_id.clone(), 0);
            }
            return Ok(0);
        }

        // If this is the first batch, lock the percentage
        if start_index == 0 {
            set_partial_refund_percentage(&env, event_id.clone(), percentage_bps);
        }
        let active_pct = get_partial_refund_percentage(&env, event_id.clone());

        let end_index = core::cmp::min(start_index + batch_size, total_payments);
        let mut processed_count = 0;
        let mut total_refunded = 0;
        let mut balance = get_event_balance(&env, event_id.clone());

        let token_address = crate::storage::get_usdc_token(&env);
        let token_client = token::Client::new(&env, &token_address);
        let contract_address = env.current_contract_address();

        for i in start_index..end_index {
            let payment_id = payment_ids.get(i).unwrap();
            if let Some(mut payment) = get_payment(&env, payment_id.clone()) {
                if payment.status == PaymentStatus::Confirmed {
                    let refund_amount = (payment
                        .amount
                        .checked_mul(active_pct as i128)
                        .ok_or(TicketPaymentError::ArithmeticError)?)
                        / (MAX_BPS as i128);

                    if refund_amount > 0 && payment.organizer_amount >= refund_amount {
                        token_client.transfer(
                            &contract_address,
                            &payment.buyer_address,
                            &refund_amount,
                        );

                        payment.refunded_amount += refund_amount;
                        payment.organizer_amount -= refund_amount;
                        store_payment(&env, payment.clone());

                        balance.organizer_amount -= refund_amount;
                        total_refunded += refund_amount;
                        processed_count += 1;
                    }
                }
            }
        }

        if processed_count > 0 {
            crate::storage::set_event_balance(&env, event_id.clone(), balance);
            subtract_from_active_escrow_total(&env, total_refunded);
            subtract_from_active_escrow_by_token(&env, token_address, total_refunded);
        }

        set_partial_refund_index(&env, event_id.clone(), end_index);

        // If finished, reset tracking
        if end_index >= total_payments {
            set_partial_refund_percentage(&env, event_id.clone(), 0);
            set_partial_refund_index(&env, event_id.clone(), 0);
        }

        // Emit partial refund event
        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::PartialRefundProcessed,),
            PartialRefundProcessedEvent {
                event_id,
                refund_count: processed_count,
                total_refunded,
                percentage_bps: active_pct,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(processed_count)
    }

    /// Protocol-wide gross ticket volume processed (all tokens combined).
    pub fn get_total_volume_processed(env: Env) -> i128 {
        crate::storage::get_total_volume_processed(&env)
    }

    /// Cumulative platform fees collected for a specific token.
    pub fn get_total_fees_collected(env: Env, token_address: Address) -> i128 {
        crate::storage::get_total_fees_collected_by_token(&env, token_address)
    }

    /// Protocol-wide active escrow liquidity (all tokens combined).
    pub fn get_active_escrow_total(env: Env) -> i128 {
        crate::storage::get_active_escrow_total(&env)
    }

    /// Active escrow liquidity for a specific token.
    pub fn get_active_escrow_total_by_token(env: Env, token_address: Address) -> i128 {
        crate::storage::get_active_escrow_by_token(&env, token_address)
    }

    pub fn get_withdrawal_cap(env: Env, token: Address) -> i128 {
        crate::storage::get_withdrawal_cap(&env, token)
    }

    pub fn get_daily_withdrawn_amount(env: Env, token: Address) -> i128 {
        let current_day = env.ledger().timestamp() / 86400;
        crate::storage::get_daily_withdrawn_amount(&env, token, current_day)
    }

    /// Places a bid for an auction tier. Escrows funds and refunds the previous highest bidder.
    pub fn place_bid(
        env: Env,
        event_id: String,
        ticket_tier_id: String,
        bidder_address: Address,
        token_address: Address,
        amount: i128,
    ) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }
        bidder_address.require_auth();

        if !is_token_whitelisted(&env, &token_address) {
            return Err(TicketPaymentError::TokenNotWhitelisted);
        }

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        if !event_info.is_active
            || matches!(event_info.status, event_registry::EventStatus::Cancelled)
        {
            return Err(TicketPaymentError::EventInactive);
        }

        let tier = event_info
            .tiers
            .get(ticket_tier_id.clone())
            .ok_or(TicketPaymentError::TierNotFound)?;

        if tier.auction_config.is_empty() {
            return Err(TicketPaymentError::NotAuctionTier);
        }
        let auction_config = tier.auction_config.get(0).unwrap();

        let current_time = env.ledger().timestamp();
        if current_time > auction_config.end_time {
            return Err(TicketPaymentError::AuctionEnded);
        }

        // Check against HighestBid
        let mut previous_bidder = None;
        let min_required = if let Some(highest_bid) =
            get_highest_bid(&env, event_id.clone(), ticket_tier_id.clone())
        {
            previous_bidder = Some(highest_bid.clone());
            highest_bid
                .amount
                .checked_add(auction_config.min_increment)
                .ok_or(TicketPaymentError::ArithmeticError)?
        } else {
            auction_config.start_price
        };

        if amount < min_required {
            return Err(TicketPaymentError::BidTooLow);
        }

        // Escrow funds
        let token_client = token::Client::new(&env, &token_address);
        let contract_address = env.current_contract_address();

        let allowance = token_client.allowance(&bidder_address, &contract_address);
        if allowance < amount {
            return Err(TicketPaymentError::InsufficientAllowance);
        }

        token_client.transfer_from(
            &contract_address,
            &bidder_address,
            &contract_address,
            &amount,
        );
        add_to_active_escrow_total(&env, amount);
        add_to_active_escrow_by_token(&env, token_address.clone(), amount);

        // Refund previous bidder if exists
        if let Some(prev) = previous_bidder {
            token_client.transfer(&contract_address, &prev.bidder, &prev.amount);
            subtract_from_active_escrow_total(&env, prev.amount);
            subtract_from_active_escrow_by_token(&env, token_address.clone(), prev.amount);
        }

        // Save new highest bid
        let new_bid = HighestBid {
            bidder: bidder_address.clone(),
            amount,
        };
        set_highest_bid(&env, event_id.clone(), ticket_tier_id.clone(), new_bid);

        env.events().publish(
            (AgoraEvent::BidPlaced,),
            BidPlacedEvent {
                event_id,
                tier_id: ticket_tier_id,
                bidder: bidder_address,
                amount,
                timestamp: current_time,
            },
        );

        Ok(())
    }

    /// Closes an auction, finalizing the highest bid and issuing exactly one ticket to the winner.
    pub fn close_auction(
        env: Env,
        payment_id: String,
        event_id: String,
        ticket_tier_id: String,
    ) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        if !event_info.is_active
            || matches!(event_info.status, event_registry::EventStatus::Cancelled)
        {
            return Err(TicketPaymentError::EventInactive);
        }

        let tier = event_info
            .tiers
            .get(ticket_tier_id.clone())
            .ok_or(TicketPaymentError::TierNotFound)?;

        if tier.auction_config.is_empty() {
            return Err(TicketPaymentError::NotAuctionTier);
        }
        let auction_config = tier.auction_config.get(0).unwrap();

        let current_time = env.ledger().timestamp();
        if current_time <= auction_config.end_time {
            return Err(TicketPaymentError::AuctionNotEnded);
        }

        if is_auction_closed(&env, event_id.clone(), ticket_tier_id.clone()) {
            return Err(TicketPaymentError::AuctionEnded); // Already closed
        }

        // Get the winning bid
        let winning_bid = get_highest_bid(&env, event_id.clone(), ticket_tier_id.clone())
            .ok_or(TicketPaymentError::NoFundsAvailable)?; // Fails if no bids

        let amount = winning_bid.amount;
        let bidder_address = winning_bid.bidder.clone();

        // Mark auction as closed to prevent double generation of tickets
        set_auction_closed(&env, event_id.clone(), ticket_tier_id.clone());

        // Platform fee calculated based on final hammer price
        let fee_bps = event_info
            .custom_fee_bps
            .unwrap_or(event_info.platform_fee_percent);

        let total_platform_fee = amount
            .checked_mul(fee_bps as i128)
            .and_then(|v| v.checked_div(MAX_BPS as i128))
            .ok_or(TicketPaymentError::ArithmeticError)?;

        let total_organizer_amount = amount
            .checked_sub(total_platform_fee)
            .ok_or(TicketPaymentError::ArithmeticError)?;

        // Update protocol fees and event balances
        let token_address = get_usdc_token(&env);

        update_event_balance(
            &env,
            event_id.clone(),
            total_organizer_amount,
            total_platform_fee,
        );
        add_to_total_volume_processed(&env, amount);
        add_to_total_fees_collected_by_token(&env, token_address.clone(), total_platform_fee);

        // Increment inventory
        registry_client.increment_inventory(
            &event_id,
            &ticket_tier_id,
            &bidder_address,
            &1,
        );

        // Record the payment
        let empty_tx_hash = String::from_str(&env, "");
        let payment = Payment {
            payment_id: payment_id.clone(),
            event_id: event_id.clone(),
            buyer_address: bidder_address.clone(),
            ticket_tier_id: ticket_tier_id.clone(),
            amount,
            platform_fee: total_platform_fee,
            organizer_amount: total_organizer_amount,
            status: PaymentStatus::Confirmed,
            transaction_hash: empty_tx_hash,
            created_at: env.ledger().timestamp(),
            confirmed_at: Some(env.ledger().timestamp()),
            refunded_amount: 0,
            is_soulbound: false,
            last_checked_in_at: 0,
        };
        store_payment(&env, payment);

        // Emit events
        env.events().publish(
            (AgoraEvent::AuctionClosed,),
            AuctionClosedEvent {
                event_id: event_id.clone(),
                tier_id: ticket_tier_id,
                winner: bidder_address.clone(),
                amount,
                timestamp: current_time,
            },
        );

        env.events().publish(
            (AgoraEvent::PaymentProcessed,),
            PaymentProcessedEvent {
                payment_id,
                event_id,
                buyer_address: bidder_address,
                amount,
                platform_fee: total_platform_fee,
                timestamp: current_time,
            },
        );

        Ok(())
    }

    /// Allows an event organizer to register a list of SHA-256 hashed discount codes.
    /// When a buyer provides the raw preimage during `process_payment`, the contract hashes
    /// it on-chain, validates against this registry, applies a 10% discount, and marks
    /// the code as used (one-time use).
    pub fn add_discount_hashes(
        env: Env,
        event_id: String,
        hashes: Vec<BytesN<32>>,
    ) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        // Only the event organizer may upload discount codes for their event
        event_info.organizer_address.require_auth();

        for hash in hashes.iter() {
            add_discount_hash(&env, hash);
        }

        Ok(())
    }

    /// Cancels an event, locking its escrow balance for refunds only.
    /// Only the event organizer can call this. Once cancelled, no new tickets can be sold.
    pub fn cancel_event(env: Env, event_id: String) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }

        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        event_info.organizer_address.require_auth();

        crate::storage::set_event_cancelled_for_refund(&env, &event_id);

        env.events().publish(
            (AgoraEvent::EventCancelled,),
            crate::events::EventCancelledEvent {
                event_id,
                organizer: event_info.organizer_address,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Returns whether an event has been locally cancelled for refunds.
    pub fn is_event_cancelled(env: Env, event_id: String) -> bool {
        crate::storage::is_event_cancelled_for_refund(&env, &event_id)
    }

    /// Allows any valid ticket holder to claim a 100% refund for a cancelled event.
    /// Skips the normal refund deadline check.
    pub fn claim_cancellation_refund(
        env: Env,
        payment_id: String,
    ) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        let mut payment =
            get_payment(&env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

        payment.buyer_address.require_auth();

        // Must be cancelled locally OR via the registry
        let is_locally_cancelled =
            crate::storage::is_event_cancelled_for_refund(&env, &payment.event_id);

        if !is_locally_cancelled {
            // Fall back to checking the registry
            let event_registry_addr = get_event_registry(&env);
            let registry_client = event_registry::Client::new(&env, &event_registry_addr);
            let event_info = match registry_client.try_get_event(&payment.event_id) {
                Ok(Ok(Some(info))) => info,
                _ => return Err(TicketPaymentError::EventNotFound),
            };
            if !matches!(event_info.status, event_registry::EventStatus::Cancelled) {
                return Err(TicketPaymentError::EventNotCompleted); // event is not cancelled
            }
        }

        if payment.status == PaymentStatus::Refunded || payment.status == PaymentStatus::Failed {
            return Err(TicketPaymentError::InvalidPaymentStatus);
        }

        let refund_amount = payment.amount;

        let old_status = payment.status.clone();
        payment.status = PaymentStatus::Refunded;
        payment.confirmed_at = Some(env.ledger().timestamp());
        payment.refunded_amount = refund_amount;

        let key = DataKey::Payment(payment_id.clone());
        env.storage().persistent().set(&key, &payment);

        if old_status != PaymentStatus::Refunded {
            crate::storage::update_payment_status_index(
                &env,
                payment.event_id.clone(),
                old_status,
                PaymentStatus::Refunded,
                payment_id.clone(),
            );
        }

        // Return ticket to inventory
        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);
        registry_client.decrement_inventory(
            &payment.event_id,
            &payment.ticket_tier_id,
            &payment.buyer_address,
        );

        // Transfer full amount back to buyer
        let token_address = crate::storage::get_usdc_token(&env);
        token::Client::new(&env, &token_address).transfer(
            &env.current_contract_address(),
            &payment.buyer_address,
            &refund_amount,
        );

        // Adjust escrow accounting
        crate::storage::update_event_balance(
            &env,
            payment.event_id.clone(),
            -payment.organizer_amount,
            -payment.platform_fee,
        );
        subtract_from_active_escrow_total(&env, refund_amount);
        subtract_from_active_escrow_by_token(&env, token_address, refund_amount);

        env.events().publish(
            (AgoraEvent::CancellationRefundClaimed,),
            crate::events::CancellationRefundClaimedEvent {
                payment_id,
                event_id: payment.event_id,
                buyer: payment.buyer_address,
                amount: refund_amount,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }
}

fn validate_address(env: &Env, address: &Address) -> Result<(), TicketPaymentError> {
    if address == &env.current_contract_address() || is_zero_address(env, address) {
        return Err(TicketPaymentError::InvalidAddress);
    }
    Ok(())
}

/// Validates that a transfer recipient is neither the zero address nor the contract itself.
fn validate_recipient(env: &Env, address: &Address) -> Result<(), TicketPaymentError> {
    if address == &env.current_contract_address() || is_zero_address(env, address) {
        return Err(TicketPaymentError::InvalidAddress);
    }
    Ok(())
}

fn is_zero_address(env: &Env, address: &Address) -> bool {
    let zero = soroban_sdk::String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJXFF",
    );
    address.to_string() == zero
}

fn fetch_fresh_asset_price(
    env: &Env,
    asset: &Address,
) -> Result<price_oracle::PriceData, TicketPaymentError> {
    let oracle_addr = get_oracle_address(env).ok_or(TicketPaymentError::OracleNotConfigured)?;
    let oracle_client = price_oracle::OracleClient::new(env, &oracle_addr);
    let price_data = oracle_client
        .lastprice(asset)
        .ok_or(TicketPaymentError::OraclePriceUnavailable)?;

    let current_time = env.ledger().timestamp();
    if current_time.saturating_sub(price_data.timestamp) > MAX_ORACLE_PRICE_AGE_SECS {
        return Err(TicketPaymentError::OraclePriceStale);
    }

    Ok(price_data)
}
