use crate::types::{DataKey, EventBalance, Payment, PaymentStatus};
use soroban_sdk::{vec, Address, Env, String, Vec};

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
        let event_key = DataKey::EventPayments(payment.event_id.clone());
        let mut event_payments: Vec<String> = env
            .storage()
            .persistent()
            .get(&event_key)
            .unwrap_or(vec![env]);
        event_payments.push_back(payment.payment_id.clone());
        env.storage().persistent().set(&event_key, &event_payments);

        // Index by buyer
        let buyer_key = DataKey::BuyerPayments(payment.buyer_address.clone());
        let mut buyer_payments: Vec<String> = env
            .storage()
            .persistent()
            .get(&buyer_key)
            .unwrap_or(vec![env]);
        buyer_payments.push_back(payment.payment_id.clone());
        env.storage().persistent().set(&buyer_key, &buyer_payments);
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
        payment.status = status;
        payment.confirmed_at = confirmed_at;
        let key = DataKey::Payment(payment_id);
        env.storage().persistent().set(&key, &payment);
    }
}

pub fn get_event_payments(env: &Env, event_id: String) -> Vec<String> {
    let key = DataKey::EventPayments(event_id);
    env.storage().persistent().get(&key).unwrap_or(vec![env])
}

pub fn get_buyer_payments(env: &Env, buyer_address: Address) -> Vec<String> {
    let key = DataKey::BuyerPayments(buyer_address);
    env.storage().persistent().get(&key).unwrap_or(vec![env])
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
    balance.organizer_amount += organizer_amount;
    balance.platform_fee += platform_fee;
    env.storage()
        .persistent()
        .set(&DataKey::Balances(event_id), &balance);
}

pub fn set_event_balance(env: &Env, event_id: String, balance: EventBalance) {
    env.storage()
        .persistent()
        .set(&DataKey::Balances(event_id), &balance);
}

pub fn set_transfer_fee(env: &Env, event_id: String, fee: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::TransferFee(event_id), &fee);
}

pub fn get_transfer_fee(env: &Env, event_id: String) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::TransferFee(event_id))
        .unwrap_or(0)
}

pub fn add_payment_to_buyer_index(env: &Env, buyer_address: Address, payment_id: String) {
    let key = DataKey::BuyerPayments(buyer_address);
    let mut buyer_payments: Vec<String> = env.storage().persistent().get(&key).unwrap_or(vec![env]);
    buyer_payments.push_back(payment_id);
    env.storage().persistent().set(&key, &buyer_payments);
}

pub fn remove_payment_from_buyer_index(env: &Env, buyer_address: Address, payment_id: String) {
    let key = DataKey::BuyerPayments(buyer_address);
    if let Some(buyer_payments) = env.storage().persistent().get::<DataKey, Vec<String>>(&key) {
        let mut new_payments = vec![env];
        for p_id in buyer_payments.iter() {
            if p_id != payment_id {
                new_payments.push_back(p_id);
            }
        }
        env.storage().persistent().set(&key, &new_payments);
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
    let total = get_total_volume_processed(env) + amount;
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
    env.storage()
        .persistent()
        .set(&DataKey::TotalFeesCollected(token), &(current + amount));
}

pub fn subtract_from_total_fees_collected_by_token(env: &Env, token: Address, amount: i128) {
    let current = get_total_fees_collected_by_token(env, token.clone());
    env.storage()
        .persistent()
        .set(&DataKey::TotalFeesCollected(token), &(current - amount));
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
        &(current + amount),
    );
}

pub fn get_active_escrow_total(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::ActiveEscrowTotal)
        .unwrap_or(0)
}

pub fn add_to_active_escrow_total(env: &Env, amount: i128) {
    let total = get_active_escrow_total(env) + amount;
    env.storage()
        .persistent()
        .set(&DataKey::ActiveEscrowTotal, &total);
}

pub fn subtract_from_active_escrow_total(env: &Env, amount: i128) {
    let total = get_active_escrow_total(env) - amount;
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
    env.storage()
        .persistent()
        .set(&DataKey::ActiveEscrowByToken(token), &(current + amount));
}

pub fn subtract_from_active_escrow_by_token(env: &Env, token: Address, amount: i128) {
    let current = get_active_escrow_by_token(env, token.clone());
    env.storage()
        .persistent()
        .set(&DataKey::ActiveEscrowByToken(token), &(current - amount));
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
