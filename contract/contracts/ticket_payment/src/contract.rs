use crate::storage::{
    add_discount_hash, add_payment_to_buyer_index, add_to_active_escrow_by_token,
    add_to_active_escrow_total, add_to_daily_withdrawn_amount,
    add_to_total_fees_collected_by_token, add_to_total_volume_processed, add_token_to_whitelist,
    get_admin, get_bulk_refund_index, get_daily_withdrawn_amount, get_event_balance,
    get_event_payments, get_event_registry, get_partial_refund_index,
    get_partial_refund_percentage, get_payment, get_platform_wallet,
    get_total_fees_collected_by_token, get_transfer_fee, get_withdrawal_cap, has_price_switched,
    is_discount_hash_used, is_discount_hash_valid, is_event_disputed, is_initialized, is_paused,
    is_token_whitelisted, mark_discount_hash_used, remove_payment_from_buyer_index,
    remove_token_from_whitelist, set_admin, set_bulk_refund_index, set_event_dispute_status,
    set_event_registry, set_initialized, set_is_paused, set_partial_refund_index,
    set_partial_refund_percentage, set_platform_wallet, set_price_switched, set_transfer_fee,
    set_usdc_token, set_withdrawal_cap, store_payment, subtract_from_active_escrow_by_token,
    subtract_from_active_escrow_total, subtract_from_total_fees_collected_by_token,
    update_event_balance,
};
use crate::types::{Payment, PaymentStatus};
use crate::{
    error::TicketPaymentError,
    events::{
        AgoraEvent, BulkRefundProcessedEvent, ContractPausedEvent, ContractUpgraded,
        DiscountCodeAppliedEvent, DisputeStatusChangedEvent, FeeSettledEvent,
        GlobalPromoAppliedEvent, InitializationEvent, PartialRefundProcessedEvent,
        PaymentProcessedEvent, PaymentStatusChangedEvent, PriceSwitchedEvent, RevenueClaimedEvent,
        TicketTransferredEvent,
    },
};
use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String, Vec};

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
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct EventInventory {
        pub current_supply: i128,
        pub max_supply: i128,
    }

    #[contractclient(name = "Client")]
    pub trait EventRegistryInterface {
        fn get_event_payment_info(env: Env, event_id: String) -> PaymentInfo;
        fn get_event(env: Env, event_id: String) -> Option<EventInfo>;
        fn increment_inventory(env: Env, event_id: String, tier_id: String, quantity: u32);
        fn decrement_inventory(env: Env, event_id: String, tier_id: String);
        fn get_global_promo_bps(env: Env) -> u32;
        fn get_promo_expiry(env: Env) -> u64;
        fn is_scanner_authorized(env: Env, event_id: String, scanner: Address) -> bool;
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct TicketTier {
        pub name: String,
        pub price: i128,
        pub early_bird_price: i128,
        pub early_bird_deadline: u64,
        pub tier_limit: i128,
        pub current_sold: i128,
        pub is_refundable: bool,
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
    }
}

#[contract]
pub struct TicketPaymentContract;

#[contractimpl]
#[allow(deprecated)]
#[allow(clippy::too_many_arguments)]
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
        let admin = get_admin(&env).ok_or(TicketPaymentError::NotInitialized)?;
        admin.require_auth();
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

    /// Sets or clears a dispute for an event. Only callable by admin.
    pub fn set_event_dispute(
        env: Env,
        event_id: String,
        disputed: bool,
    ) -> Result<(), TicketPaymentError> {
        let admin = get_admin(&env).ok_or(TicketPaymentError::NotInitialized)?;
        admin.require_auth();

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
        let admin = get_admin(&env).expect("Admin not set");
        admin.require_auth();

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
    }

    pub fn add_token(env: Env, token: Address) {
        let admin = get_admin(&env).expect("Admin not set");
        admin.require_auth();
        add_token_to_whitelist(&env, &token);
    }

    pub fn remove_token(env: Env, token: Address) {
        let admin = get_admin(&env).expect("Admin not set");
        admin.require_auth();
        remove_token_from_whitelist(&env, &token);
    }

    pub fn is_token_allowed(env: Env, token: Address) -> bool {
        is_token_whitelisted(&env, &token)
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
                .checked_mul((10000 - global_promo_bps as i128) as i128)
                .and_then(|v| v.checked_div(10000))
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
                return Err(TicketPaymentError::DiscountCodeAlreadyUsed);
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

        let tier = event_info
            .tiers
            .get(ticket_tier_id.clone())
            .ok_or(TicketPaymentError::TierNotFound)?;

        let current_time = env.ledger().timestamp();
        let mut active_price = tier.price;

        if tier.early_bird_deadline > 0 && current_time <= tier.early_bird_deadline {
            active_price = tier.early_bird_price;
        }

        if amount != active_price {
            return Err(TicketPaymentError::InvalidPrice);
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

        // 2. Calculate platform fee (platform_fee_percent is in bps, 10000 = 100%)
        let mut total_platform_fee = effective_total
            .checked_mul(event_info.platform_fee_percent as i128)
            .and_then(|v| v.checked_div(10000))
            .ok_or(TicketPaymentError::ArithmeticError)?;
        let total_organizer_amount = effective_total
            .checked_sub(total_platform_fee)
            .ok_or(TicketPaymentError::ArithmeticError)?;

        let referral_reward = if referrer.is_some() {
            let reward = total_platform_fee
                .checked_mul(20)
                .and_then(|v| v.checked_div(100))
                .ok_or(TicketPaymentError::ArithmeticError)?; // 20%
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
        registry_client.increment_inventory(&event_id, &ticket_tier_id, &quantity);

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
            };

            store_payment(&env, payment);
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
        let admin = get_admin(&env).expect("Admin not set");
        admin.require_auth();
        // In a real scenario, this would be restricted to a specific backend/admin address.
        if let Some(mut payment) = get_payment(&env, payment_id.clone()) {
            payment.status = PaymentStatus::Confirmed;
            payment.confirmed_at = Some(env.ledger().timestamp());
            payment.transaction_hash = transaction_hash.clone();
            store_payment(&env, payment);
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

    /// Triggers a refund as an administrator, regardless of dispute status.
    pub fn admin_refund(env: Env, payment_id: String) -> Result<(), TicketPaymentError> {
        let admin = get_admin(&env).ok_or(TicketPaymentError::NotInitialized)?;
        admin.require_auth();

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

        // Ensure the event is cancelled for automatic refund
        if !matches!(event_info.status, event_registry::EventStatus::Cancelled) {
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

        // Check if refundable or if EVENT IS CANCELLED
        if !tier.is_refundable && !is_cancelled && event_info.is_active {
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
        // Bypass restocking fee if the event is cancelled.
        let effective_restocking_fee = if is_cancelled {
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
        registry_client.decrement_inventory(&payment.event_id, &payment.ticket_tier_id);

        let old_status = payment.status.clone();
        payment.status = PaymentStatus::Refunded;
        payment.confirmed_at = Some(env.ledger().timestamp());

        store_payment(&env, payment.clone());

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

    /// Verifies scanner authorization and marks a ticket as CheckedIn.
    pub fn check_in(
        env: Env,
        payment_id: String,
        scanner: Address,
    ) -> Result<(), TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        if is_paused(&env) {
            return Err(TicketPaymentError::ContractPaused);
        }

        let mut payment =
            get_payment(&env, payment_id.clone()).ok_or(TicketPaymentError::PaymentNotFound)?;

        // Must authenticate the scanner wallet calling this entry point
        scanner.require_auth();

        if payment.status == PaymentStatus::CheckedIn {
            return Err(TicketPaymentError::TicketAlreadyUsed);
        }

        // Verify scanner authorization
        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);
        let is_auth = registry_client.is_scanner_authorized(&payment.event_id, &scanner);
        if !is_auth {
            return Err(TicketPaymentError::UnauthorizedScanner);
        }

        // Update status and store arrival timestamp
        payment.status = PaymentStatus::CheckedIn;
        payment.confirmed_at = Some(env.ledger().timestamp());

        store_payment(&env, payment.clone());

        #[allow(deprecated)]
        env.events().publish(
            (AgoraEvent::TicketCheckedIn,),
            crate::events::TicketCheckedInEvent {
                payment_id,
                event_id: payment.event_id,
                scanner,
                timestamp: env.ledger().timestamp(),
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

        let total_revenue = balance
            .organizer_amount
            .checked_add(balance.total_withdrawn)
            .ok_or(TicketPaymentError::ArithmeticError)?;
        if total_revenue == 0 {
            return Ok(0);
        }

        let mut release_percent = 10000u32;
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
            .and_then(|v| v.checked_div(10000))
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
        let admin = get_admin(&env).ok_or(TicketPaymentError::NotInitialized)?;
        admin.require_auth();

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
        let admin = get_admin(&env).ok_or(TicketPaymentError::NotInitialized)?;
        admin.require_auth();

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
        let admin = get_admin(&env).ok_or(TicketPaymentError::NotInitialized)?;
        admin.require_auth();

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

        if event_info.is_active {
            return Err(TicketPaymentError::EventNotCompleted);
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
                    fee_bps: event_info.platform_fee_percent,
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

    /// Sets the transfer fee for an event. Only the organizer can call this.
    pub fn set_transfer_fee(
        env: Env,
        event_id: String,
        amount: i128,
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

        if amount < 0 {
            panic!("Transfer fee must be non-negative");
        }

        set_transfer_fee(&env, event_id, amount);
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

        let from = payment.buyer_address.clone();
        from.require_auth();

        if from == to {
            return Err(TicketPaymentError::InvalidAddress);
        }

        // Validate resale price against the organizer's cap
        if let Some(price) = sale_price {
            let event_registry_addr = get_event_registry(&env);
            let registry_client = event_registry::Client::new(&env, &event_registry_addr);

            if let Some(event_info) = registry_client.get_event(&payment.event_id) {
                if let Some(cap_bps) = event_info.resale_cap_bps {
                    // Look up the original tier face-value price
                    let tier = event_info
                        .tiers
                        .get(payment.ticket_tier_id.clone())
                        .ok_or(TicketPaymentError::TierNotFound)?;
                    let original_price = tier.price;

                    // max_price = original_price * (10000 + cap_bps) / 10000
                    let max_price = original_price
                        .checked_mul(
                            (10000i128)
                                .checked_add(cap_bps as i128)
                                .unwrap_or(i128::MAX),
                        )
                        .ok_or(TicketPaymentError::ArithmeticError)?
                        / 10000;

                    if price > max_price {
                        return Err(TicketPaymentError::ResalePriceExceedsCap);
                    }
                }
            }
        }

        let transfer_fee = get_transfer_fee(&env, payment.event_id.clone());

        if transfer_fee > 0 {
            let token_address = crate::storage::get_usdc_token(&env);
            let token_client = token::Client::new(&env, &token_address);
            let contract_address = env.current_contract_address();

            // Transfer fee from old owner to contract
            token_client.transfer_from(&contract_address, &from, &contract_address, &transfer_fee);

            // Update escrow balances (fee goes to organizer)
            update_event_balance(&env, payment.event_id.clone(), transfer_fee, 0);
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
                to,
                transfer_fee,
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

        // In a bulk refund, we assume the event is cancelled or inactive
        if event_info.is_active
            && !matches!(event_info.status, event_registry::EventStatus::Cancelled)
        {
            // Bulk refund is typically for cancelled events or post-event settlements.
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
        if percentage_bps > 10000 {
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
                        / 10000;

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
}

fn validate_address(env: &Env, address: &Address) -> Result<(), TicketPaymentError> {
    if address == &env.current_contract_address() {
        return Err(TicketPaymentError::InvalidAddress);
    }
    Ok(())
}
