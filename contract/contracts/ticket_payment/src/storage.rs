use crate::{
    error::TicketPaymentError,
    types::{
        DataKey, DiscountData, EventBalance, HighestBid, ParameterProposal, Payment, PaymentStatus,
    },
};
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String, Vec};

const SHARD_SIZE: u32 = 100;

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

pub fn store_payment(env: &Env, payment: Payment) {
    let key = DataKey::Payment(payment.payment_id.clone());
    let exists = env.storage().persistent().has(&key);

    env.storage().persistent().set(&key, &payment);

    if !exists {
        // Index by event
        add_payment_to_event_index(env, payment.event_id.clone(), payment.payment_id.clone());

        // Index by buyer
        add_payment_to_buyer_index(
            env,
            payment.buyer_address.clone(),
            payment.payment_id.clone(),
        );

        // Index by status
        add_payment_to_status_index(
            env,
            payment.event_id.clone(),
            payment.status.clone(),
            payment.payment_id.clone(),
        );
    }
}

pub fn get_payment(env: &Env, payment_id: String) -> Option<Payment> {
    let key = DataKey::Payment(payment_id);
    env.storage().persistent().get(&key)
}

pub fn update_payment_status(
    env: &Env,
    payment_id: String,
    status: PaymentStatus,
    confirmed_at: Option<u64>,
) {
    if let Some(mut payment) = get_payment(env, payment_id.clone()) {
        let old_status = payment.status.clone();
        payment.status = status.clone();
        payment.confirmed_at = confirmed_at;
        let key = DataKey::Payment(payment_id.clone());
        env.storage().persistent().set(&key, &payment);

        // Update status index if status changed
        if old_status != status {
            update_payment_status_index(
                env,
                payment.event_id.clone(),
                old_status,
                status,
                payment_id,
            );
        }
    }
}

pub fn get_event_payment_count(env: &Env, event_id: String) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::EventPaymentCount(event_id))
        .unwrap_or(0)
}

pub fn get_event_payments(env: &Env, event_id: String) -> Vec<String> {
    let count = get_event_payment_count(env, event_id.clone());
    let mut all_payments = vec![env];

    if count == 0 {
        return all_payments;
    }

    let num_shards = count.div_ceil(SHARD_SIZE);
    for i in 0..num_shards {
        let shard: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::EventPaymentShard(event_id.clone(), i))
            .unwrap_or_else(|| vec![env]);
        for id in shard.iter() {
            all_payments.push_back(id);
        }
    }
    all_payments
}

pub fn get_buyer_payment_count(env: &Env, buyer_address: Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::BuyerPaymentCount(buyer_address))
        .unwrap_or(0)
}

pub fn get_buyer_payments(env: &Env, buyer_address: Address) -> Vec<String> {
    let count = get_buyer_payment_count(env, buyer_address.clone());
    let mut all_payments = vec![env];

    if count == 0 {
        return all_payments;
    }

    let num_shards = count.div_ceil(SHARD_SIZE);
    for i in 0..num_shards {
        let shard: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::BuyerPaymentShard(buyer_address.clone(), i))
            .unwrap_or_else(|| vec![env]);
        for id in shard.iter() {
            all_payments.push_back(id);
        }
    }
    all_payments
}

// Configuration getters/setters
pub fn set_usdc_token(env: &Env, address: Address) {
    env.storage()
        .persistent()
        .set(&DataKey::UsdcToken, &address);
}

pub fn get_usdc_token(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::UsdcToken)
        .expect("USDC token not set")
}

pub fn set_platform_wallet(env: &Env, address: Address) {
    env.storage()
        .persistent()
        .set(&DataKey::PlatformWallet, &address);
}

pub fn get_platform_wallet(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::PlatformWallet)
        .expect("Platform wallet not set")
}

pub fn set_event_registry(env: &Env, address: Address) {
    env.storage()
        .persistent()
        .set(&DataKey::EventRegistry, &address);
}

pub fn get_event_registry(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::EventRegistry)
        .expect("Event registry not set")
}

pub fn set_initialized(env: &Env, value: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::Initialized, &value);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Initialized)
        .unwrap_or(false)
}

pub fn set_is_paused(env: &Env, paused: bool) {
    env.storage().persistent().set(&DataKey::IsPaused, &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

pub fn add_token_to_whitelist(env: &Env, token: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::TokenWhitelist(token.clone()), &true);
}

pub fn remove_token_from_whitelist(env: &Env, token: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::TokenWhitelist(token.clone()));
}

pub fn is_token_whitelisted(env: &Env, token: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::TokenWhitelist(token.clone()))
        .unwrap_or(false)
}

pub fn get_event_balance(env: &Env, event_id: String) -> EventBalance {
    env.storage()
        .persistent()
        .get(&DataKey::Balances(event_id))
        .unwrap_or(EventBalance {
            organizer_amount: 0,
            total_withdrawn: 0,
            platform_fee: 0,
        })
}

pub fn update_event_balance(
    env: &Env,
    event_id: String,
    organizer_amount: i128,
    platform_fee: i128,
) {
    let mut balance = get_event_balance(env, event_id.clone());
    balance.organizer_amount = balance
        .organizer_amount
        .checked_add(organizer_amount)
        .unwrap();
    balance.platform_fee = balance.platform_fee.checked_add(platform_fee).unwrap();
    env.storage()
        .persistent()
        .set(&DataKey::Balances(event_id), &balance);
}

pub fn set_event_balance(env: &Env, event_id: String, balance: EventBalance) {
    env.storage()
        .persistent()
        .set(&DataKey::Balances(event_id), &balance);
}

pub fn set_transfer_fee(env: &Env, event_id: String, fee: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::TransferFee(event_id), &fee);
}

pub fn get_transfer_fee(env: &Env, event_id: String) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::TransferFee(event_id))
}

pub fn add_payment_to_event_index(env: &Env, event_id: String, payment_id: String) {
    if env
        .storage()
        .persistent()
        .has(&DataKey::EventPayment(event_id.clone(), payment_id.clone()))
    {
        return;
    }

    let count = get_event_payment_count(env, event_id.clone());
    let shard_id = count / SHARD_SIZE;

    let mut shard: Vec<String> = env
        .storage()
        .persistent()
        .get(&DataKey::EventPaymentShard(event_id.clone(), shard_id))
        .unwrap_or_else(|| vec![env]);

    shard.push_back(payment_id.clone());
    env.storage().persistent().set(
        &DataKey::EventPaymentShard(event_id.clone(), shard_id),
        &shard,
    );

    env.storage()
        .persistent()
        .set(&DataKey::EventPaymentCount(event_id.clone()), &(count + 1));

    env.storage()
        .persistent()
        .set(&DataKey::EventPayment(event_id, payment_id), &true);
}

pub fn add_payment_to_buyer_index(env: &Env, buyer_address: Address, payment_id: String) {
    if env.storage().persistent().has(&DataKey::BuyerPayment(
        buyer_address.clone(),
        payment_id.clone(),
    )) {
        return;
    }

    let count = get_buyer_payment_count(env, buyer_address.clone());
    let shard_id = count / SHARD_SIZE;

    let mut shard: Vec<String> = env
        .storage()
        .persistent()
        .get(&DataKey::BuyerPaymentShard(buyer_address.clone(), shard_id))
        .unwrap_or_else(|| vec![env]);

    shard.push_back(payment_id.clone());
    env.storage().persistent().set(
        &DataKey::BuyerPaymentShard(buyer_address.clone(), shard_id),
        &shard,
    );

    env.storage().persistent().set(
        &DataKey::BuyerPaymentCount(buyer_address.clone()),
        &(count + 1),
    );

    env.storage()
        .persistent()
        .set(&DataKey::BuyerPayment(buyer_address, payment_id), &true);
}

pub fn remove_payment_from_buyer_index(env: &Env, buyer_address: Address, payment_id: String) {
    // Note: Removal from sharded lists is GAS-INTENSIVE as it requires finding the shard.
    // However, we maintain it for correctness of transfer_ticket.
    let count = get_buyer_payment_count(env, buyer_address.clone());
    if count == 0 {
        return;
    }

    let num_shards = count.div_ceil(SHARD_SIZE);
    let mut found = false;

    for i in 0..num_shards {
        let shard: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::BuyerPaymentShard(buyer_address.clone(), i))
            .unwrap_or_else(|| vec![env]);

        let mut found_in_shard = false;
        let mut new_shard = vec![env];

        for p_id in shard.iter() {
            if p_id == payment_id {
                found_in_shard = true;
                found = true;
            } else {
                new_shard.push_back(p_id);
            }
        }

        if found_in_shard {
            env.storage().persistent().set(
                &DataKey::BuyerPaymentShard(buyer_address.clone(), i),
                &new_shard,
            );
            // We break here assuming payment_id is unique per buyer
            break;
        }
    }

    if found {
        env.storage().persistent().set(
            &DataKey::BuyerPaymentCount(buyer_address.clone()),
            &(count - 1),
        );
        env.storage()
            .persistent()
            .remove(&DataKey::BuyerPayment(buyer_address, payment_id));
    }
}

pub fn set_bulk_refund_index(env: &Env, event_id: String, index: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::BulkRefundIndex(event_id), &index);
}

pub fn get_bulk_refund_index(env: &Env, event_id: String) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::BulkRefundIndex(event_id))
        .unwrap_or(0)
}

pub fn set_partial_refund_index(env: &Env, event_id: String, index: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::PartialRefundIndex(event_id), &index);
}

pub fn get_partial_refund_index(env: &Env, event_id: String) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::PartialRefundIndex(event_id))
        .unwrap_or(0)
}

pub fn set_partial_refund_percentage(env: &Env, event_id: String, percentage_bps: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::PartialRefundPercentage(event_id), &percentage_bps);
}

pub fn get_partial_refund_percentage(env: &Env, event_id: String) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::PartialRefundPercentage(event_id))
        .unwrap_or(0)
}

pub fn has_price_switched(env: &Env, event_id: String, tier_id: String) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::PriceSwitched(event_id, tier_id))
        .unwrap_or(false)
}

pub fn set_price_switched(env: &Env, event_id: String, tier_id: String) {
    env.storage()
        .persistent()
        .set(&DataKey::PriceSwitched(event_id, tier_id), &true);
}

pub fn get_total_volume_processed(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalVolumeProcessed)
        .unwrap_or(0)
}

pub fn add_to_total_volume_processed(env: &Env, amount: i128) {
    let total = get_total_volume_processed(env).checked_add(amount).unwrap();
    env.storage()
        .persistent()
        .set(&DataKey::TotalVolumeProcessed, &total);
}

pub fn get_total_fees_collected_by_token(env: &Env, token: Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalFeesCollected(token))
        .unwrap_or(0)
}

pub fn add_to_total_fees_collected_by_token(env: &Env, token: Address, amount: i128) {
    let current = get_total_fees_collected_by_token(env, token.clone());
    env.storage().persistent().set(
        &DataKey::TotalFeesCollected(token),
        &current.checked_add(amount).unwrap(),
    );
}

pub fn subtract_from_total_fees_collected_by_token(env: &Env, token: Address, amount: i128) {
    let current = get_total_fees_collected_by_token(env, token.clone());
    env.storage().persistent().set(
        &DataKey::TotalFeesCollected(token),
        &current.checked_sub(amount).unwrap(),
    );
}

pub fn set_withdrawal_cap(env: &Env, token: Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::WithdrawalCap(token), &amount);
}

pub fn get_withdrawal_cap(env: &Env, token: Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::WithdrawalCap(token))
        .unwrap_or(0)
}

pub fn get_daily_withdrawn_amount(env: &Env, token: Address, day: u64) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::DailyWithdrawalAmount(token, day))
        .unwrap_or(0)
}

pub fn add_to_daily_withdrawn_amount(env: &Env, token: Address, day: u64, amount: i128) {
    let current = get_daily_withdrawn_amount(env, token.clone(), day);
    env.storage().persistent().set(
        &DataKey::DailyWithdrawalAmount(token, day),
        &current.checked_add(amount).unwrap(),
    );
}

pub fn get_active_escrow_total(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::ActiveEscrowTotal)
        .unwrap_or(0)
}

pub fn add_to_active_escrow_total(env: &Env, amount: i128) {
    let total = get_active_escrow_total(env).checked_add(amount).unwrap();
    env.storage()
        .persistent()
        .set(&DataKey::ActiveEscrowTotal, &total);
}

pub fn subtract_from_active_escrow_total(env: &Env, amount: i128) {
    let total = get_active_escrow_total(env).checked_sub(amount).unwrap();
    env.storage()
        .persistent()
        .set(&DataKey::ActiveEscrowTotal, &total);
}

pub fn get_active_escrow_by_token(env: &Env, token: Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::ActiveEscrowByToken(token))
        .unwrap_or(0)
}

pub fn add_to_active_escrow_by_token(env: &Env, token: Address, amount: i128) {
    let current = get_active_escrow_by_token(env, token.clone());
    env.storage().persistent().set(
        &DataKey::ActiveEscrowByToken(token),
        &current.checked_add(amount).unwrap(),
    );
}

pub fn subtract_from_active_escrow_by_token(env: &Env, token: Address, amount: i128) {
    let current = get_active_escrow_by_token(env, token.clone());
    env.storage().persistent().set(
        &DataKey::ActiveEscrowByToken(token),
        &current.checked_sub(amount).unwrap(),
    );
}

// ── Discount code registry ────────────────────────────────────────────────────

/// Register a SHA-256 hash as a valid (unused) discount code.
pub fn add_discount_hash(env: &Env, hash: soroban_sdk::BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::DiscountCodeHash(hash), &true);
}

/// Returns `true` if the hash has been registered as a discount code.
pub fn is_discount_hash_valid(env: &Env, hash: &soroban_sdk::BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::DiscountCodeHash(hash.clone()))
        .unwrap_or(false)
}

/// Returns `true` if the hash has already been redeemed.
pub fn is_discount_hash_used(env: &Env, hash: &soroban_sdk::BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::DiscountCodeUsed(hash.clone()))
        .unwrap_or(false)
}

/// Mark a discount code hash as spent so it cannot be reused.
pub fn mark_discount_hash_used(env: &Env, hash: soroban_sdk::BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::DiscountCodeUsed(hash), &true);
}

pub fn is_event_disputed(env: &Env, event_id: String) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::DisputeStatus(event_id))
        .unwrap_or(false)
}

pub fn set_event_dispute_status(env: &Env, event_id: String, disputed: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::DisputeStatus(event_id), &disputed);
}

pub fn is_event_cancelled_for_refund(env: &Env, event_id: &String) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::EventCancelledForRefund(event_id.clone()))
        .unwrap_or(false)
}

pub fn set_event_cancelled_for_refund(env: &Env, event_id: &String) {
    env.storage()
        .persistent()
        .set(&DataKey::EventCancelledForRefund(event_id.clone()), &true);
}

// ── Oracle configuration ──────────────────────────────────────────────────────

pub fn set_oracle_address(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::OracleAddress, address);
}

pub fn get_oracle_address(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::OracleAddress)
}

pub fn set_slippage_bps(env: &Env, bps: u32) -> Result<(), TicketPaymentError> {
    if bps > 5000 {
        return Err(TicketPaymentError::InvalidSlippageBps);
    }

    env.storage().persistent().set(&DataKey::SlippageBps, &bps);
    Ok(())
}

pub fn get_slippage_bps(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::SlippageBps)
        .unwrap_or(200)
}

// ── Auction functions ─────────────────────────────────────────────────────────

pub fn set_highest_bid(env: &Env, event_id: String, tier_id: String, bid: HighestBid) {
    env.storage()
        .persistent()
        .set(&DataKey::HighestBid(event_id, tier_id), &bid);
}

pub fn get_highest_bid(env: &Env, event_id: String, tier_id: String) -> Option<HighestBid> {
    env.storage()
        .persistent()
        .get(&DataKey::HighestBid(event_id, tier_id))
}

pub fn set_auction_closed(env: &Env, event_id: String, tier_id: String) {
    env.storage()
        .persistent()
        .set(&DataKey::AuctionClosed(event_id, tier_id), &true);
}

pub fn is_auction_closed(env: &Env, event_id: String, tier_id: String) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::AuctionClosed(event_id, tier_id))
        .unwrap_or(false)
}

// ── Governance functions ──────────────────────────────────────────────────────

pub fn is_governor(env: &Env, address: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Governor(address.clone()))
        .unwrap_or(false)
}

pub fn set_governor(env: &Env, address: &Address, status: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::Governor(address.clone()), &status);
}

pub fn get_total_governors(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalGovernors)
        .unwrap_or(0)
}

pub fn set_total_governors(env: &Env, total: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::TotalGovernors, &total);
}

pub fn get_proposal(env: &Env, id: u64) -> Option<ParameterProposal> {
    env.storage().persistent().get(&DataKey::Proposal(id))
}

pub fn set_proposal(env: &Env, proposal: &ParameterProposal) {
    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal.id), proposal);
}

pub fn get_proposal_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::ProposalCount)
        .unwrap_or(0)
}

pub fn increment_proposal_count(env: &Env) -> u64 {
    let count = get_proposal_count(env) + 1;
    env.storage()
        .persistent()
        .set(&DataKey::ProposalCount, &count);
    count
}

// ── Payment Status Index ──────────────────────────────────────────────────────

/// Adds a payment to the status index for an event
pub fn add_payment_to_status_index(
    env: &Env,
    event_id: String,
    status: PaymentStatus,
    payment_id: String,
) {
    // Check if already indexed
    if env
        .storage()
        .persistent()
        .has(&DataKey::EventPaymentStatusEntry(
            event_id.clone(),
            status.clone(),
            payment_id.clone(),
        ))
    {
        return;
    }

    // Get current list for this event and status
    let mut payment_ids: Vec<String> = env
        .storage()
        .persistent()
        .get(&DataKey::EventPaymentStatus(
            event_id.clone(),
            status.clone(),
        ))
        .unwrap_or_else(|| vec![env]);

    // Add payment_id to the list
    payment_ids.push_back(payment_id.clone());

    // Store updated list
    env.storage().persistent().set(
        &DataKey::EventPaymentStatus(event_id.clone(), status.clone()),
        &payment_ids,
    );

    // Mark as indexed
    env.storage().persistent().set(
        &DataKey::EventPaymentStatusEntry(event_id, status, payment_id),
        &true,
    );
}

/// Removes a payment from the status index (used when status changes)
pub fn remove_payment_from_status_index(
    env: &Env,
    event_id: String,
    status: PaymentStatus,
    payment_id: String,
) {
    // Get current list for this event and status
    let payment_ids: Vec<String> = env
        .storage()
        .persistent()
        .get(&DataKey::EventPaymentStatus(
            event_id.clone(),
            status.clone(),
        ))
        .unwrap_or_else(|| vec![env]);

    // Filter out the payment_id
    let mut new_payment_ids = vec![env];
    for id in payment_ids.iter() {
        if id != payment_id {
            new_payment_ids.push_back(id);
        }
    }

    // Store updated list
    env.storage().persistent().set(
        &DataKey::EventPaymentStatus(event_id.clone(), status.clone()),
        &new_payment_ids,
    );

    // Remove index marker
    env.storage()
        .persistent()
        .remove(&DataKey::EventPaymentStatusEntry(
            event_id, status, payment_id,
        ));
}

/// Updates the status index when a payment status changes
pub fn update_payment_status_index(
    env: &Env,
    event_id: String,
    old_status: PaymentStatus,
    new_status: PaymentStatus,
    payment_id: String,
) {
    // Remove from old status index
    remove_payment_from_status_index(env, event_id.clone(), old_status, payment_id.clone());

    // Add to new status index
    add_payment_to_status_index(env, event_id, new_status, payment_id);
}

/// Gets all payment IDs for an event with a specific status
pub fn get_payments_by_status(env: &Env, event_id: String, status: PaymentStatus) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&DataKey::EventPaymentStatus(event_id, status))
        .unwrap_or_else(|| vec![env])
}

/// Stores the SHA-256 hash of the ticket secret for a payment.
pub fn store_validation_hash(env: &Env, payment_id: &String, hash: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::ValidationHash(payment_id.clone()), hash);
}

/// Retrieves the stored validation hash for a payment.
pub fn get_validation_hash(env: &Env, payment_id: &String) -> Option<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::ValidationHash(payment_id.clone()))
}

/// Verifies that `raw_secret` hashes to the stored validation hash.
pub fn verify_secret(env: &Env, payment_id: &String, raw_secret: &Bytes) -> bool {
    match get_validation_hash(env, payment_id) {
        Some(stored_hash) => {
            let computed: BytesN<32> = env.crypto().sha256(raw_secret).into();
            computed == stored_hash
        }
        None => false,
    }
}

// ── Per-event discount codes ──────────────────────────────────────────────────

pub fn set_discount_code(env: &Env, event_id: String, code: String, data: &DiscountData) {
    env.storage()
        .persistent()
        .set(&DataKey::DiscountCode(event_id, code), data);
}

pub fn get_discount_code(env: &Env, event_id: &String, code: &String) -> Option<DiscountData> {
    env.storage()
        .persistent()
        .get(&DataKey::DiscountCode(event_id.clone(), code.clone()))
}

// ── Affiliate commission rates ────────────────────────────────────────────────

/// Sets a per-event affiliate commission rate in basis points.
/// Only rates in [1, 10000] are meaningful; 0 means "use default".
pub fn set_affiliate_rate(env: &Env, event_id: String, affiliate: &Address, rate_bps: u32) {
    env.storage().persistent().set(
        &DataKey::AffiliateRate(event_id, affiliate.clone()),
        &rate_bps,
    );
}

/// Returns the affiliate-specific commission rate for (event_id, affiliate), if set.
pub fn get_affiliate_rate(env: &Env, event_id: &String, affiliate: &Address) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::AffiliateRate(event_id.clone(), affiliate.clone()))
}
