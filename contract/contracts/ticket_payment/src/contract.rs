use crate::storage::{
    add_token_to_whitelist, get_admin, get_event_registry, get_payment, get_platform_wallet,
    is_initialized, is_token_whitelisted, remove_token_from_whitelist, set_admin,
    set_event_registry, set_initialized, set_platform_wallet, set_usdc_token, store_payment,
    update_payment_status,
};
use crate::types::{Payment, PaymentStatus};
use crate::{
    error::TicketPaymentError,
    events::{
        AgoraEvent, ContractUpgraded, InitializationEvent, PaymentProcessedEvent,
        PaymentStatusChangedEvent,
    },
};
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, String};

// Event Registry interface
pub mod event_registry {
    use soroban_sdk::{contractclient, Address, Env, String};

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
        fn increment_inventory(env: Env, event_id: String, tier_id: String);
        fn decrement_inventory(env: Env, event_id: String, tier_id: String);
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct TicketTier {
        pub name: String,
        pub price: i128,
        pub tier_limit: i128,
        pub current_sold: i128,
        pub is_refundable: bool,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct EventInfo {
        pub event_id: String,
        pub organizer_address: Address,
        pub payment_address: Address,
        pub platform_fee_percent: u32,
        pub is_active: bool,
        pub created_at: u64,
        pub metadata_cid: String,
        pub max_supply: i128,
        pub current_supply: i128,
        pub tiers: soroban_sdk::Map<String, TicketTier>,
        pub custom_fee_bps: Option<u32>,
    }
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
        set_usdc_token(&env, usdc_token.clone());
        set_platform_wallet(&env, platform_wallet.clone());
        set_event_registry(&env, event_registry.clone());
        set_initialized(&env, true);

        // Whitelist USDC by default
        add_token_to_whitelist(&env, &usdc_token);

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

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin(&env).expect("Admin not set");
        admin.require_auth();

        let old_wasm_hash = match env.current_contract_address().executable() {
            Some(soroban_sdk::Executable::Wasm(hash)) => hash,
            _ => panic!("Current contract is not a Wasm contract"),
        };

        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());

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
    pub fn process_payment(
        env: Env,
        payment_id: String,
        event_id: String,
        ticket_tier_id: String,
        buyer_address: Address,
        token_address: Address,
        amount: i128,
    ) -> Result<String, TicketPaymentError> {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        buyer_address.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        if !is_token_whitelisted(&env, &token_address) {
            return Err(TicketPaymentError::TokenNotWhitelisted);
        }

        // 1. Query Event Registry for event info and check inventory
        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);

        let event_info = match registry_client.try_get_event(&event_id) {
            Ok(Ok(Some(info))) => info,
            Ok(Ok(None)) => return Err(TicketPaymentError::EventNotFound),
            _ => return Err(TicketPaymentError::EventNotFound),
        };

        if !event_info.is_active {
            return Err(TicketPaymentError::EventInactive);
        }

        // Check if tickets are available (max_supply of 0 means unlimited)
        if event_info.max_supply > 0 && event_info.current_supply >= event_info.max_supply {
            return Err(TicketPaymentError::MaxSupplyExceeded);
        }

        // 2. Calculate platform fee using custom fee if set, otherwise use default
        let fee_bps = event_info
            .custom_fee_bps
            .unwrap_or(event_info.platform_fee_percent);
        let platform_fee = (amount * fee_bps as i128) / 10000;
        let organizer_amount = amount - platform_fee;

        // 3. Transfer tokens from buyer (splitting payment)
        let token_client = token::Client::new(&env, &token_address);
        let platform_wallet = get_platform_wallet(&env);

        // Transfer platform fee
        if platform_fee > 0 {
            token_client.transfer(&buyer_address, &platform_wallet, &platform_fee);
        }

        // Transfer organizer amount
        if organizer_amount > 0 {
            token_client.transfer(
                &buyer_address,
                &event_info.payment_address,
                &organizer_amount,
            );
        }

        // 4. Increment inventory after successful payment
        registry_client.increment_inventory(&event_id, &ticket_tier_id);

        // 5. Create payment record
        let payment = Payment {
            payment_id: payment_id.clone(),
            event_id: event_id.clone(),
            buyer_address: buyer_address.clone(),
            ticket_tier_id,
            amount,
            platform_fee,
            organizer_amount,
            status: PaymentStatus::Pending,
            transaction_hash: String::from_str(&env, ""), // Empty until confirmed
            created_at: env.ledger().timestamp(),
            confirmed_at: None,
        };

        store_payment(&env, payment);

        // 6. Emit payment event
        env.events().publish(
            (AgoraEvent::PaymentProcessed,),
            PaymentProcessedEvent {
                payment_id: payment_id.clone(),
                event_id: event_id.clone(),
                buyer_address: buyer_address.clone(),
                amount,
                platform_fee,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(payment_id)
    }

    /// Confirms a payment after backend verification.
    pub fn confirm_payment(env: Env, payment_id: String, transaction_hash: String) {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        // In a real scenario, this would be restricted to a specific backend/admin address.
        update_payment_status(
            &env,
            payment_id.clone(),
            PaymentStatus::Confirmed,
            Some(env.ledger().timestamp()),
        );

        // Update the transaction hash
        if let Some(mut payment) = get_payment(&env, payment_id.clone()) {
            payment.transaction_hash = transaction_hash.clone();
            store_payment(&env, payment);
        }

        // Emit confirmation event
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

        // Check if refundable or if EVENT IS CANCELLED (is_active == false)
        if !tier.is_refundable && event_info.is_active {
            return Err(TicketPaymentError::TicketNotRefundable);
        }

        // Return ticket to inventory using the authorized contract interface
        registry_client.decrement_inventory(&payment.event_id, &payment.ticket_tier_id);

        let old_status = payment.status.clone();
        payment.status = PaymentStatus::Refunded;
        payment.confirmed_at = Some(env.ledger().timestamp());

        store_payment(&env, payment);

        // Emit confirmation event
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

    /// Returns the status and details of a payment.
    pub fn get_payment_status(env: Env, payment_id: String) -> Option<Payment> {
        get_payment(&env, payment_id)
    }
}

fn validate_address(env: &Env, address: &Address) -> Result<(), TicketPaymentError> {
    if address == &env.current_contract_address() {
        return Err(TicketPaymentError::InvalidAddress);
    }
    Ok(())
}
