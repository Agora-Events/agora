use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, String};

use crate::{
    error::ProSubscriptionError,
    events::{
        AdminUpdatedEvent, ContractUpgradedEvent, InitializationEvent, PriceUpdatedEvent,
        ProMemberAddedEvent, ProMemberRemovedEvent, ProSubscriptionEvent,
        SubscriptionCancelledEvent, SubscriptionCreatedEvent, SubscriptionExpiredEvent,
        SubscriptionRenewedEvent,
    },
    storage::{
        add_to_pro_members_list, decrement_total_pro_subscriptions, get_admin, get_payment_token,
        get_platform_wallet, get_pro_members_list, get_pro_monthly_price, get_subscription,
        get_total_pro_subscriptions, increment_total_pro_subscriptions, is_initialized,
        remove_from_pro_members_list, set_admin, set_initialized, set_payment_token,
        set_platform_wallet, set_pro_monthly_price, set_subscription,
    },
    types::{
        Subscription, SubscriptionTier, DISCOUNT_BPS_12_MONTHS, DISCOUNT_BPS_3_MONTHS,
        DISCOUNT_BPS_6_MONTHS, MAX_BPS, MAX_SUBSCRIPTION_MONTHS, SECONDS_PER_MONTH,
    },
    validation::validate_address,
};

fn require_admin(env: &Env) -> Result<Address, ProSubscriptionError> {
    let admin = get_admin(env).ok_or(ProSubscriptionError::NotInitialized)?;
    admin.require_auth();
    Ok(admin)
}

/// Returns the total price for `months` of Pro, applying the multi-month discount tier.
fn calculate_subscription_price(
    monthly_price: i128,
    months: u32,
) -> Result<i128, ProSubscriptionError> {
    let discount_bps = if months >= 12 {
        DISCOUNT_BPS_12_MONTHS
    } else if months >= 6 {
        DISCOUNT_BPS_6_MONTHS
    } else if months >= 3 {
        DISCOUNT_BPS_3_MONTHS
    } else {
        0
    };

    monthly_price
        .checked_mul(months as i128)
        .and_then(|v| v.checked_mul((MAX_BPS - discount_bps) as i128))
        .and_then(|v| v.checked_div(MAX_BPS as i128))
        .ok_or(ProSubscriptionError::ArithmeticError)
}

/// Returns the stored subscription without mutating state.
///
/// This is a pure read: it never writes storage and never emits events. Use
/// [`ProSubscriptionContract::sync_expired`] to persist the expiry cleanup.
fn get_subscription_with_expiration(env: &Env, organizer: &Address) -> Option<Subscription> {
    get_subscription(env, organizer)
}

/// Validates the requested subscription duration in months.
fn validate_months(months: u32) -> Result<(), ProSubscriptionError> {
    if months == 0 || months > MAX_SUBSCRIPTION_MONTHS {
        return Err(ProSubscriptionError::InvalidDuration);
    }
    Ok(())
}

#[contract]
pub struct ProSubscriptionContract;

#[contractimpl]
impl ProSubscriptionContract {
    /// Initializes the contract with admin, platform wallet, payment token, and pricing
    pub fn initialize(
        env: Env,
        admin: Address,
        platform_wallet: Address,
        payment_token: Address,
        pro_monthly_price: i128,
    ) -> Result<(), ProSubscriptionError> {
        if is_initialized(&env) {
            return Err(ProSubscriptionError::AlreadyInitialized);
        }

        validate_address(&env, &admin)?;
        validate_address(&env, &platform_wallet)?;
        validate_address(&env, &payment_token)?;

        if pro_monthly_price <= 0 {
            return Err(ProSubscriptionError::InvalidPrice);
        }

        set_admin(&env, &admin);
        set_platform_wallet(&env, &platform_wallet);
        set_payment_token(&env, &payment_token);
        set_pro_monthly_price(&env, pro_monthly_price);
        set_initialized(&env, true);

        env.events().publish(
            (ProSubscriptionEvent::ContractInitialized,),
            InitializationEvent {
                admin,
                platform_wallet,
                payment_token,
                pro_monthly_price,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Returns the version of the contract as a String.
    ///
    /// The version is sourced from the package version at compile time via `env!("CARGO_PKG_VERSION")`.
    /// This allows verification of which build is deployed at a given contract address without
    /// diffing WASM hashes.
    pub fn version(_env: Env) -> String {
        String::from_str(&_env, env!("CARGO_PKG_VERSION"))
    }

    /// Subscribe an organizer to the Pro tier
    /// The organizer must approve the contract to spend the subscription fee
    pub fn subscribe_pro(
        env: Env,
        organizer: Address,
        months: u32,
    ) -> Result<(), ProSubscriptionError> {
        if !is_initialized(&env) {
            return Err(ProSubscriptionError::NotInitialized);
        }

        organizer.require_auth();

        validate_months(months)?;

        // Persist any pending expiry cleanup before checking the current state.
        Self::sync_expired(env.clone(), organizer.clone())?;

        // Check if already has an active subscription
        if let Some(existing) = get_subscription_with_expiration(&env, &organizer) {
            if existing.is_active && existing.expires_at > env.ledger().timestamp() {
                return Err(ProSubscriptionError::SubscriptionAlreadyActive);
            }
        }

        let monthly_price = get_pro_monthly_price(&env);
        let total_amount = calculate_subscription_price(monthly_price, months)?;

        let payment_token = get_payment_token(&env).ok_or(ProSubscriptionError::NotInitialized)?;
        let platform_wallet =
            get_platform_wallet(&env).ok_or(ProSubscriptionError::NotInitialized)?;

        // Transfer payment from organizer to platform wallet
        let token_client = token::Client::new(&env, &payment_token);
        token_client.transfer(&organizer, &platform_wallet, &total_amount);

        let current_time = env.ledger().timestamp();
        let expires_at = current_time
            .checked_add(SECONDS_PER_MONTH * months as u64)
            .ok_or(ProSubscriptionError::ArithmeticError)?;

        let subscription = Subscription {
            organizer: organizer.clone(),
            tier: SubscriptionTier::Pro,
            started_at: current_time,
            expires_at,
            is_active: true,
            amount_paid: total_amount,
            payment_token: payment_token.clone(),
        };

        set_subscription(&env, &subscription);
        add_to_pro_members_list(&env, &organizer);
        increment_total_pro_subscriptions(&env);

        env.events().publish(
            (ProSubscriptionEvent::ProMemberAdded,),
            ProMemberAddedEvent {
                organizer: organizer.clone(),
                timestamp: current_time,
            },
        );

        env.events().publish(
            (ProSubscriptionEvent::SubscriptionCreated,),
            SubscriptionCreatedEvent {
                organizer,
                tier: SubscriptionTier::Pro,
                amount_paid: total_amount,
                expires_at,
                timestamp: current_time,
            },
        );

        Ok(())
    }

    /// Renew an existing Pro subscription
    pub fn renew_subscription(
        env: Env,
        organizer: Address,
        months: u32,
    ) -> Result<(), ProSubscriptionError> {
        if !is_initialized(&env) {
            return Err(ProSubscriptionError::NotInitialized);
        }

        organizer.require_auth();

        validate_months(months)?;

        // Persist any pending expiry cleanup before renewing.
        Self::sync_expired(env.clone(), organizer.clone())?;

        let mut subscription =
            get_subscription(&env, &organizer).ok_or(ProSubscriptionError::SubscriptionNotFound)?;

        let monthly_price = get_pro_monthly_price(&env);
        let total_amount = calculate_subscription_price(monthly_price, months)?;

        let payment_token = get_payment_token(&env).ok_or(ProSubscriptionError::NotInitialized)?;
        let platform_wallet =
            get_platform_wallet(&env).ok_or(ProSubscriptionError::NotInitialized)?;

        // Transfer payment from organizer to platform wallet
        let token_client = token::Client::new(&env, &payment_token);
        token_client.transfer(&organizer, &platform_wallet, &total_amount);

        let current_time = env.ledger().timestamp();

        // If subscription is expired, start from now; otherwise extend from current expiry
        let base_time = if subscription.expires_at < current_time {
            current_time
        } else {
            subscription.expires_at
        };

        let new_expiry = base_time
            .checked_add(SECONDS_PER_MONTH * months as u64)
            .ok_or(ProSubscriptionError::ArithmeticError)?;

        subscription.expires_at = new_expiry;
        subscription.is_active = true;
        subscription.amount_paid = subscription
            .amount_paid
            .checked_add(total_amount)
            .ok_or(ProSubscriptionError::ArithmeticError)?;

        set_subscription(&env, &subscription);

        // Ensure they're in the pro members list
        add_to_pro_members_list(&env, &organizer);

        env.events().publish(
            (ProSubscriptionEvent::ProMemberAdded,),
            ProMemberAddedEvent {
                organizer: organizer.clone(),
                timestamp: current_time,
            },
        );

        env.events().publish(
            (ProSubscriptionEvent::SubscriptionRenewed,),
            SubscriptionRenewedEvent {
                organizer,
                amount_paid: total_amount,
                new_expiry,
                timestamp: current_time,
            },
        );

        Ok(())
    }

    /// Cancel a subscription (admin only)
    pub fn cancel_subscription(env: Env, organizer: Address) -> Result<(), ProSubscriptionError> {
        let admin = require_admin(&env)?;

        Self::sync_expired(env.clone(), organizer.clone())?;

        let Some(mut subscription) = get_subscription_with_expiration(&env, &organizer) else {
            return Err(ProSubscriptionError::SubscriptionNotFound);
        };
        if !subscription.is_active {
            return Ok(());
        }

        subscription.is_active = false;
        set_subscription(&env, &subscription);
        remove_from_pro_members_list(&env, &organizer);
        decrement_total_pro_subscriptions(&env);

        env.events().publish(
            (ProSubscriptionEvent::ProMemberRemoved,),
            ProMemberRemovedEvent {
                organizer: organizer.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );

        env.events().publish(
            (ProSubscriptionEvent::SubscriptionCancelled,),
            SubscriptionCancelledEvent {
                organizer,
                cancelled_by: admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Cancel the caller's own active Pro subscription.
    pub fn cancel_my_subscription(
        env: Env,
        organizer: Address,
    ) -> Result<(), ProSubscriptionError> {
        if !is_initialized(&env) {
            return Err(ProSubscriptionError::NotInitialized);
        }
        organizer.require_auth();

        Self::sync_expired(env.clone(), organizer.clone())?;

        let Some(mut subscription) = get_subscription_with_expiration(&env, &organizer) else {
            return Err(ProSubscriptionError::SubscriptionNotFound);
        };
        if !subscription.is_active {
            return Ok(());
        }
        if subscription.tier != SubscriptionTier::Pro {
            return Err(ProSubscriptionError::SubscriptionNotFound);
        }

        subscription.is_active = false;
        set_subscription(&env, &subscription);
        remove_from_pro_members_list(&env, &organizer);
        decrement_total_pro_subscriptions(&env);

        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (ProSubscriptionEvent::ProMemberRemoved,),
            ProMemberRemovedEvent {
                organizer: organizer.clone(),
                timestamp,
            },
        );
        env.events().publish(
            (ProSubscriptionEvent::SubscriptionCancelled,),
            SubscriptionCancelledEvent {
                organizer: organizer.clone(),
                cancelled_by: organizer,
                timestamp,
            },
        );

        Ok(())
    }

    /// Check if an organizer is a pro member with an active, non-expired subscription.
    ///
    /// This getter is side-effect free: it performs no storage writes and emits no events.
    pub fn is_pro_member(env: Env, organizer: Address) -> bool {
        if let Some(subscription) = get_subscription_with_expiration(&env, &organizer) {
            subscription.is_active && subscription.expires_at > env.ledger().timestamp()
        } else {
            false
        }
    }

    /// Get subscription details for an organizer.
    ///
    /// This getter is side-effect free: it performs no storage writes and emits no events.
    pub fn get_subscription(env: Env, organizer: Address) -> Option<Subscription> {
        get_subscription_with_expiration(&env, &organizer)
    }

    /// Get subscription expiry timestamp for an organizer.
    ///
    /// This getter is side-effect free: it performs no storage writes and emits no events.
    pub fn get_subscription_expiry(env: Env, organizer: Address) -> Option<u64> {
        get_subscription_with_expiration(&env, &organizer).map(|s| s.expires_at)
    }

    /// Persist the cleanup of a lapsed Pro subscription for `organizer`.
    ///
    /// If the organizer's Pro subscription is active but past its expiry, this marks it
    /// inactive, removes the organizer from the members list, decrements the total counter
    /// and publishes a `SubscriptionExpired` event. Anyone may call this; it is a no-op when
    /// there is nothing to clean up.
    pub fn sync_expired(env: Env, organizer: Address) -> Result<(), ProSubscriptionError> {
        let Some(mut subscription) = get_subscription(&env, &organizer) else {
            return Ok(());
        };
        if subscription.tier == SubscriptionTier::Pro
            && subscription.is_active
            && subscription.expires_at <= env.ledger().timestamp()
        {
            subscription.is_active = false;
            set_subscription(&env, &subscription);
            remove_from_pro_members_list(&env, &organizer);
            decrement_total_pro_subscriptions(&env);
            env.events().publish(
                (ProSubscriptionEvent::SubscriptionExpired,),
                SubscriptionExpiredEvent {
                    organizer: organizer.clone(),
                    expires_at: subscription.expires_at,
                    timestamp: env.ledger().timestamp(),
                },
            );
        }
        Ok(())
    }

    /// Get all active pro members
    pub fn get_pro_members(env: Env) -> soroban_sdk::Vec<Address> {
        get_pro_members_list(&env)
    }

    /// Get count of pro members in the members list
    pub fn get_pro_members_count(env: Env) -> u32 {
        get_pro_members_list(&env).len()
    }

    /// Register a Basic (free) subscription for an organizer
    pub fn register_basic(env: Env, organizer: Address) -> Result<(), ProSubscriptionError> {
        if !is_initialized(&env) {
            return Err(ProSubscriptionError::NotInitialized);
        }
        organizer.require_auth();
        if let Some(existing) = get_subscription(&env, &organizer) {
            if existing.tier == SubscriptionTier::Pro && existing.is_active {
                return Err(ProSubscriptionError::SubscriptionAlreadyActive);
            }
        }
        let subscription = Subscription {
            organizer: organizer.clone(),
            tier: SubscriptionTier::Basic,
            started_at: env.ledger().timestamp(),
            expires_at: 0,
            is_active: true,
            amount_paid: 0,
            payment_token: get_payment_token(&env).unwrap_or(env.current_contract_address()),
        };
        set_subscription(&env, &subscription);
        Ok(())
    }

    /// Get total count of active pro subscriptions
    pub fn get_total_pro_subscriptions(env: Env) -> u32 {
        get_total_pro_subscriptions(&env)
    }

    /// Update the monthly price for Pro subscriptions (admin only)
    pub fn update_pro_price(env: Env, new_price: i128) -> Result<(), ProSubscriptionError> {
        let admin = require_admin(&env)?;

        if new_price <= 0 {
            return Err(ProSubscriptionError::InvalidPrice);
        }

        let old_price = get_pro_monthly_price(&env);
        set_pro_monthly_price(&env, new_price);

        env.events().publish(
            (ProSubscriptionEvent::PriceUpdated,),
            PriceUpdatedEvent {
                old_price,
                new_price,
                updated_by: admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Get the current monthly price for Pro subscriptions
    pub fn get_pro_monthly_price(env: Env) -> i128 {
        get_pro_monthly_price(&env)
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Option<Address> {
        get_admin(&env)
    }

    /// Check whether the contract has been initialized
    pub fn is_initialized(env: Env) -> bool {
        is_initialized(&env)
    }

    /// Update the admin address (admin only)
    ///
    /// Emits an `AdminUpdated` event carrying the old admin, the new admin,
    /// who performed the update, and the ledger timestamp.
    pub fn update_admin(env: Env, new_admin: Address) -> Result<(), ProSubscriptionError> {
        let old_admin = require_admin(&env)?;
        validate_address(&env, &new_admin)?;
        if new_admin == current_admin {
            return Err(ProSubscriptionError::SameAdmin);
        }
        set_admin(&env, &new_admin);

        env.events().publish(
            (ProSubscriptionEvent::AdminUpdated,),
            AdminUpdatedEvent {
                old_admin: old_admin.clone(),
                new_admin,
                updated_by: old_admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Get the platform wallet address
    pub fn get_platform_wallet(env: Env) -> Option<Address> {
        get_platform_wallet(&env)
    }

    /// Update the platform wallet address (admin only).
    ///
    /// Emits a [`ProSubscriptionEvent::PlatformWalletUpdated`] event containing the old and new
    /// wallet addresses so indexers and the backend can track payout destination changes.
    pub fn update_platform_wallet(
        env: Env,
        new_wallet: Address,
    ) -> Result<(), ProSubscriptionError> {
        let admin = require_admin(&env)?;
        validate_address(&env, &new_wallet)?;
        let old_wallet =
            get_platform_wallet(&env).ok_or(ProSubscriptionError::NotInitialized)?;
        set_platform_wallet(&env, &new_wallet);

        env.events().publish(
            (ProSubscriptionEvent::PlatformWalletUpdated,),
            PlatformWalletUpdatedEvent {
                old_wallet,
                new_wallet,
                updated_by: admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Get the payment token address
    pub fn get_payment_token(env: Env) -> Option<Address> {
        get_payment_token(&env)
    }

    /// Upgrade the contract WASM in place (admin only).
    ///
    /// Emits a [`ProSubscriptionEvent::ContractUpgraded`] event carrying the old and new
    /// WASM hashes so indexers and clients can track the deployed implementation.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ProSubscriptionError> {
        let admin = require_admin(&env)?;

        let old_wasm_hash = env.deployer().current_contract_wasm_hash();
        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());

        env.events().publish(
            (ProSubscriptionEvent::ContractUpgraded,),
            ContractUpgradedEvent {
                old_wasm_hash,
                new_wasm_hash,
                updated_by: admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Update the accepted payment token (admin only).
    ///
    /// Emits a [`ProSubscriptionEvent::PaymentTokenUpdated`] event containing the old and new
    /// token addresses so the frontend and indexer can build transactions with the correct asset.
    pub fn update_payment_token(env: Env, new_token: Address) -> Result<(), ProSubscriptionError> {
        let admin = require_admin(&env)?;
        validate_address(&env, &new_token)?;
        let old_token =
            get_payment_token(&env).ok_or(ProSubscriptionError::NotInitialized)?;
        set_payment_token(&env, &new_token);

        env.events().publish(
            (ProSubscriptionEvent::PaymentTokenUpdated,),
            PaymentTokenUpdatedEvent {
                old_token,
                new_token,
                updated_by: admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }
}
