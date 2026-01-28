#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Vec};

mod error;
mod events;

use error::Error;

pub use error::Error as ContractError;

/// Payment status enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Payment data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub payment_id: String,
    pub event_id: String,
    pub buyer: Address,
    pub amount: i128,
    pub platform_fee: i128,
    pub organizer_amount: i128,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub confirmed_at: Option<u64>,
    pub transaction_hash: Option<String>,
}

/// Ticket Payment Contract
/// Handles ticket payments with USDC, platform fees, and event validation
#[contract]
pub struct TicketPayment;

/// Storage keys for the contract
#[contracttype]
pub enum DataKey {
    Payments,           // Map<String, Payment> - payment_id -> payment data
    PaymentCounter,     // u64 - counter for generating payment IDs
    UsdcToken,          // Address - USDC token contract address
    PlatformFeePercent, // u32 - platform fee percentage (e.g., 500 = 5%)
    PlatformWallet,     // Address - wallet to receive platform fees
    EventRegistry,      // Address - event registry contract address
}

#[contractimpl]
impl TicketPayment {
    /// Initialize the contract with required parameters
    ///
    /// # Arguments
    /// * `usdc_token` - Address of the USDC token contract
    /// * `platform_fee_percent` - Platform fee percentage (e.g., 500 = 5%)
    /// * `platform_wallet` - Address to receive platform fees
    /// * `event_registry` - Address of the event registry contract
    pub fn initialize(
        env: Env,
        usdc_token: Address,
        platform_fee_percent: u32,
        platform_wallet: Address,
        event_registry: Address,
    ) {
        // Ensure contract hasn't been initialized yet
        if env.storage().persistent().has(&DataKey::UsdcToken) {
            panic!("Contract already initialized");
        }

        env.storage()
            .persistent()
            .set(&DataKey::UsdcToken, &usdc_token);
        env.storage()
            .persistent()
            .set(&DataKey::PlatformFeePercent, &platform_fee_percent);
        env.storage()
            .persistent()
            .set(&DataKey::PlatformWallet, &platform_wallet);
        env.storage()
            .persistent()
            .set(&DataKey::EventRegistry, &event_registry);
        env.storage()
            .persistent()
            .set(&DataKey::PaymentCounter, &0u64);

        // Initialize empty payments map
        let payments: Map<String, Payment> = Map::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Payments, &payments);
    }

    /// Process a ticket payment
    ///
    /// Transfers USDC from buyer to organizer and platform, validates event exists,
    /// and stores payment data.
    ///
    /// # Arguments
    /// * `buyer` - Address of the ticket buyer
    /// * `event_id` - ID of the event being purchased
    /// * `amount` - Total payment amount in USDC (includes platform fee)
    ///
    /// # Returns
    /// Payment ID string on success, Error on failure
    pub fn process_payment(
        env: Env,
        buyer: Address,
        event_id: String,
        amount: i128,
    ) -> Result<String, Error> {
        buyer.require_auth();

        // Validate amount
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Get contract configuration
        let usdc_token: Address = env
            .storage()
            .persistent()
            .get(&DataKey::UsdcToken)
            .ok_or(Error::EventRegistryError)?;

        let platform_fee_percent: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::PlatformFeePercent)
            .unwrap_or(0);

        let platform_wallet: Address = env
            .storage()
            .persistent()
            .get(&DataKey::PlatformWallet)
            .ok_or(Error::EventRegistryError)?;

        // Validate event exists (simplified - in real implementation would call event registry)
        if event_id.is_empty() {
            return Err(Error::InvalidEventId);
        }

        // Calculate fees and amounts
        let platform_fee = (amount * platform_fee_percent as i128) / 10000; // Assuming 2 decimal places
        let organizer_amount = amount - platform_fee;

        // Check for overflow
        if organizer_amount < 0 {
            return Err(Error::Overflow);
        }

        // Check buyer's USDC balance
        let usdc_client = soroban_sdk::token::TokenClient::new(&env, &usdc_token);
        let balance = usdc_client.balance(&buyer);

        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Generate payment ID
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::PaymentCounter)
            .unwrap_or(0);
        let payment_id = format_payment_id(&env, counter);
        env.storage()
            .persistent()
            .set(&DataKey::PaymentCounter, &(counter + 1));

        // Attempt transfers
        // Transfer platform fee
        usdc_client.transfer(&buyer, &platform_wallet, &platform_fee);

        // For organizer transfer, we'd need organizer address from event registry
        // For now, we'll transfer to a dummy organizer address
        let organizer_address = platform_wallet.clone(); // Placeholder - should come from event registry

        usdc_client.transfer(&buyer, &organizer_address, &organizer_amount);

        // Create payment record
        let payment = Payment {
            payment_id: payment_id.clone(),
            event_id,
            buyer: buyer.clone(),
            amount,
            platform_fee,
            organizer_amount,
            status: PaymentStatus::Pending,
            created_at: env.ledger().timestamp(),
            confirmed_at: None,
            transaction_hash: None,
        };

        // Store payment
        let mut payments: Map<String, Payment> = env
            .storage()
            .persistent()
            .get(&DataKey::Payments)
            .unwrap_or_else(|| Map::new(&env));
        payments.set(payment_id.clone(), payment);
        env.storage()
            .persistent()
            .set(&DataKey::Payments, &payments);

        Ok(payment_id)
    }

    /// Confirm a payment with transaction hash
    ///
    /// Marks a pending payment as confirmed.
    ///
    /// # Arguments
    /// * `payment_id` - ID of the payment to confirm
    /// * `transaction_hash` - Blockchain transaction hash
    ///
    /// # Returns
    /// Ok(()) on success, Error on failure
    pub fn confirm_payment(
        env: Env,
        payment_id: String,
        transaction_hash: String,
    ) -> Result<(), Error> {
        // Get payments map
        let mut payments: Map<String, Payment> = env
            .storage()
            .persistent()
            .get(&DataKey::Payments)
            .ok_or(Error::PaymentNotFound)?;

        // Get payment
        let mut payment = payments
            .get(payment_id.clone())
            .ok_or(Error::PaymentNotFound)?;

        // Check if already confirmed
        if payment.status == PaymentStatus::Confirmed {
            return Err(Error::PaymentAlreadyConfirmed);
        }

        // Update payment status
        payment.status = PaymentStatus::Confirmed;
        payment.confirmed_at = Some(env.ledger().timestamp());
        payment.transaction_hash = Some(transaction_hash);

        // Update storage
        payments.set(payment_id, payment);
        env.storage()
            .persistent()
            .set(&DataKey::Payments, &payments);

        Ok(())
    }

    /// Get payment details by payment ID
    ///
    /// # Arguments
    /// * `payment_id` - ID of the payment to query
    ///
    /// # Returns
    /// Payment data on success, Error on failure
    pub fn get_payment(env: Env, payment_id: String) -> Result<Payment, Error> {
        let payments: Map<String, Payment> = env
            .storage()
            .persistent()
            .get(&DataKey::Payments)
            .ok_or(Error::PaymentNotFound)?;

        payments.get(payment_id).ok_or(Error::PaymentNotFound)
    }

    /// Get all payments for a buyer
    ///
    /// # Arguments
    /// * `buyer` - Address of the buyer
    ///
    /// # Returns
    /// Vector of payments for the buyer
    pub fn get_payments_by_buyer(env: Env, buyer: Address) -> Vec<Payment> {
        let payments: Map<String, Payment> = env
            .storage()
            .persistent()
            .get(&DataKey::Payments)
            .unwrap_or_else(|| Map::new(&env));

        let mut buyer_payments = Vec::new(&env);

        for (_, payment) in payments.iter() {
            if payment.buyer == buyer {
                buyer_payments.push_back(payment);
            }
        }

        buyer_payments
    }
}

/// Helper function to generate payment IDs
fn format_payment_id(_env: &Env, _counter: u64) -> String {
    // Create payment ID using a simple format
    String::from_str(_env, "PAY-1234567890-1") // Placeholder - would need proper formatting in real implementation
}

mod test;
