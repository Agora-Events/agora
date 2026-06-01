use soroban_sdk::{Address, Env, Vec};

use crate::types::{DataKey, Subscription};

const DAY_IN_LEDGERS: u32 = 17280;
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS * 30; // 30 days
const INSTANCE_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS * 60; // 60 days

// ── Admin & Configuration ──────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Initialized)
        .unwrap_or(false)
}

pub fn set_initialized(env: &Env, initialized: bool) {
    env.storage()
        .instance()
        .set(&DataKey::Initialized, &initialized);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_platform_wallet(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PlatformWallet)
}

pub fn set_platform_wallet(env: &Env, wallet: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::PlatformWallet, wallet);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_payment_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PaymentToken)
}

pub fn set_payment_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::PaymentToken, token);
    env.storage()
        .instance()
        .set(&DataKey::PaymentToken, token);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_pro_monthly_price(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::ProMonthlyPrice)
        .unwrap_or(0)
}

pub fn set_pro_monthly_price(env: &Env, price: i128) {
    env.storage()
        .instance()
        .set(&DataKey::ProMonthlyPrice, &price);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

// ── Subscription Management ────────────────────────────────────────────────

pub fn get_subscription(env: &Env, organizer: &Address) -> Option<Subscription> {
    let key = DataKey::Subscription(organizer.clone());
    env.storage().persistent().get(&key)
}

pub fn set_subscription(env: &Env, subscription: &Subscription) {
    let key = DataKey::Subscription(subscription.organizer.clone());
    env.storage().persistent().set(&key, subscription);
    env.storage()
        .persistent()
        .extend_ttl(&key, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

#[allow(dead_code)]
pub fn remove_subscription(env: &Env, organizer: &Address) {
    let key = DataKey::Subscription(organizer.clone());
    env.storage().persistent().remove(&key);
}

// ── Pro Members List ───────────────────────────────────────────────────────

pub fn get_pro_members_list(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::ProMembersList)
        .unwrap_or(Vec::new(env))
}

pub fn add_to_pro_members_list(env: &Env, organizer: &Address) {
    let mut list = get_pro_members_list(env);
    if !list.contains(organizer) {
        list.push_back(organizer.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ProMembersList, &list);
        env.storage().persistent().extend_ttl(
            &DataKey::ProMembersList,
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );
    }
}

pub fn remove_from_pro_members_list(env: &Env, organizer: &Address) {
    let mut list = get_pro_members_list(env);
    if let Some(index) = list.iter().position(|addr| addr == *organizer) {
        list.remove(index as u32);
        env.storage()
            .persistent()
            .set(&DataKey::ProMembersList, &list);
        env.storage().persistent().extend_ttl(
            &DataKey::ProMembersList,
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );
    }
}

pub fn get_total_pro_subscriptions(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalProSubscriptions)
        .unwrap_or(0)
}

pub fn increment_total_pro_subscriptions(env: &Env) {
    let count = get_total_pro_subscriptions(env) + 1;
    env.storage()
        .persistent()
        .set(&DataKey::TotalProSubscriptions, &count);
    env.storage().persistent().extend_ttl(
        &DataKey::TotalProSubscriptions,
        INSTANCE_LIFETIME_THRESHOLD,
        INSTANCE_BUMP_AMOUNT,
    );
}

pub fn decrement_total_pro_subscriptions(env: &Env) {
    let count = get_total_pro_subscriptions(env).saturating_sub(1);
    env.storage()
        .persistent()
        .set(&DataKey::TotalProSubscriptions, &count);
    env.storage().persistent().extend_ttl(
        &DataKey::TotalProSubscriptions,
        INSTANCE_LIFETIME_THRESHOLD,
        INSTANCE_BUMP_AMOUNT,
    );
}
