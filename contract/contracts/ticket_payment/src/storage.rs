use crate::types::{DataKey, Payment, PaymentStatus};
use soroban_sdk::{vec, Address, Env, String, Vec};

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

pub fn store_payment(env: &Env, payment: Payment) {
    let key = DataKey::Payment(payment.payment_id.clone());
    env.storage().persistent().set(&key, &payment);

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
