use crate::storage::{
    get_admin, get_event_registry, get_payment, get_platform_wallet, get_usdc_token, is_initialized,
    set_admin, set_event_registry, set_initialized, set_platform_wallet, set_usdc_token,
    store_payment, update_payment_status,
};
use crate::types::{Payment, PaymentStatus};
use crate::{
    error::TicketPaymentError,
    events::{ContractUpgraded, InitializationEvent},
};
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, BytesN, Env, String};

// Event Registry interface
pub mod event_registry {
    use soroban_sdk::{contractclient, Address, Env, String};

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct PaymentInfo {
        pub payment_address: Address,
        pub platform_fee_percent: u32,
    }

    #[contractclient(name = "Client")]
    pub trait EventRegistryInterface {
        fn get_event_payment_info(env: Env, event_id: String) -> PaymentInfo;
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

        InitializationEvent {
            usdc_token,
            platform_wallet,
            event_registry,
        }
        .publish(&env);

        Ok(())
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin(&env).expect("Admin not set");
        admin.require_auth();

        let old_wasm_hash = match env.current_contract_address().executable() {
            Some(soroban_sdk::Executable::Wasm(hash)) => hash,
            _ => panic!("Current contract is not a Wasm contract"),
        };

        env.deployer().update_current_contract_wasm(new_wasm_hash.clone());

        ContractUpgraded {
            old_wasm_hash,
            new_wasm_hash,
        }
        .publish(&env);
    }

    /// Processes a payment for an event ticket.
    pub fn process_payment(
        env: Env,
        payment_id: String,
        event_id: String,
        ticket_tier_id: String,
        buyer_address: Address,
        amount: i128,
    ) -> String {
        if !is_initialized(&env) {
            panic!("Contract not initialized");
        }
        buyer_address.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // 1. Query Event Registry for payment info and platform fee
        let event_registry_addr = get_event_registry(&env);
        let registry_client = event_registry::Client::new(&env, &event_registry_addr);
        let payment_info = registry_client.get_event_payment_info(&event_id);

        // 2. Calculate platform fee (platform_fee_percent is in bps, 10000 = 100%)
        let platform_fee = (amount * payment_info.platform_fee_percent as i128) / 10000;
        let organizer_amount = amount - platform_fee;

        // 3. Transfer USDC from buyer (splitting payment)
        let usdc_addr = get_usdc_token(&env);
        let token_client = token::Client::new(&env, &usdc_addr);
        let platform_wallet = get_platform_wallet(&env);

        // Transfer platform fee
        if platform_fee > 0 {
            token_client.transfer(&buyer_address, &platform_wallet, &platform_fee);
        }

        // Transfer organizer amount
        if organizer_amount > 0 {
            token_client.transfer(
                &buyer_address,
                &payment_info.payment_address,
                &organizer_amount,
            );
        }

        // 4. Create payment record
        let payment = Payment {
            payment_id: payment_id.clone(),
            event_id,
            buyer_address,
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

        // 5. Emit payment event
        env.events().publish(
            (symbol_short!("pay_proc"), payment_id.clone()),
            (amount, platform_fee),
        );

        payment_id
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
        env.events()
            .publish((symbol_short!("pay_conf"), payment_id), (transaction_hash,));
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
