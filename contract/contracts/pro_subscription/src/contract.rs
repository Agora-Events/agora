use soroban_sdk::{contract, contractimpl, token, Address, Env};

use crate::{
    error::ProSubscriptionError,
    events::{
        InitializationEvent, PriceUpdatedEvent, ProSubscriptionEvent, SubscriptionCancelledEvent,
        SubscriptionCreatedEvent, SubscriptionRenewedEvent,
    },
    storage::{
        add_to_pro_members_list, decrement_total_pro_subscriptions, get_admin,
        get_payment_token, get_platform_wallet, get_pro_members_list, get_pro_monthly_price,
        get_subscription, get_total_pro_subscriptions, increment_total_pro_subscriptions,
        is_initialized, remove_from_pro_members_list, remove_subscription, set_admin,
        set_initialized, set_payment_token, set_platform_wallet, set_pro_monthly_price,
        set_subscription,
    },
    types::{Subscription, SubscriptionTier},
    validation::validate_address,
};

const SECONDS_PER_MONTH: u64 = 30 * 24 * 60 * 60; // 30 days

fn require_admin(env: &Env) -> Result<Address, ProSubscriptionError> {
    let admin = get_admin(env).ok_or(ProSubscriptionError::NotInitialized)?;
    admin.require_auth();
    Ok(admin)
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

        if months == 0 {
            return Err(ProSubscriptionError::InvalidPrice);
        }

        // Check if already has an active subscription
        if let Some(existing) = get_subscription(&env, &organizer) {
            if existing.is_active && existing.expires_at > env.ledger().timestamp() {
                return Err(ProSubscriptionError::SubscriptionAlreadyActive);
            }
        }

        let monthly_price = get_pro_monthly_price(&env);
        let total_amount = monthly_price
            .checked_mul(months as i128)
            .ok_or(ProSubscriptionError::ArithmeticError)?;

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

        if months == 0 {
            return Err(ProSubscriptionError::InvalidPrice);
        }

        let mut subscription =
            get_subscription(&env, &organizer).ok_or(ProSubscriptionError::SubscriptionNotFound)?;

        let monthly_price = get_pro_monthly_price(&env);
        let total_amount = monthly_price
            .checked_mul(months as i128)
            .ok_or(ProSubscriptionError::ArithmeticError)?;

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
    pub fn cancel_subscription(
        env: Env,
        organizer: Address,
    ) -> Result<(), ProSubscriptionError> {
        let admin = require_admin(&env)?;

        let mut subscription =
            get_subscription(&env, &organizer).ok_or(ProSubscriptionError::SubscriptionNotFound)?;

        subscription.is_active = false;
        set_subscription(&env, &subscription);
        remove_from_pro_members_list(&env, &organizer);
        decrement_total_pro_subscriptions(&env);

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

    /// Check if an organizer is a pro member with an active, non-expired subscription
    pub fn is_pro_member(env: Env, organizer: Address) -> bool {
        if let Some(subscription) = get_subscription(&env, &organizer) {
            subscription.is_active && subscription.expires_at > env.ledger().timestamp()
        } else {
            false
        }
    }

    /// Get subscription details for an organizer
    pub fn get_subscription(env: Env, organizer: Address) -> Option<Subscription> {
        get_subscription(&env, &organizer)
    }

    /// Get all active pro members
    pub fn get_pro_members(env: Env) -> soroban_sdk::Vec<Address> {
        get_pro_members_list(&env)
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

    /// Update the admin address (admin only)
    pub fn update_admin(env: Env, new_admin: Address) -> Result<(), ProSubscriptionError> {
        require_admin(&env)?;
        validate_address(&env, &new_admin)?;
        set_admin(&env, &new_admin);
        Ok(())
    }

    /// Get the platform wallet address
    pub fn get_platform_wallet(env: Env) -> Option<Address> {
        get_platform_wallet(&env)
    }

    /// Get the payment token address
    pub fn get_payment_token(env: Env) -> Option<Address> {
        get_payment_token(&env)
    }
}
