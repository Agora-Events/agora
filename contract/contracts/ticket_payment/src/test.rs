use super::contract::TicketPaymentContract;
use super::storage::*;
use super::types::{Payment, PaymentStatus};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_store_and_get_payment() {
    let env = Env::default();
    let contract_id = env.register(TicketPaymentContract, ());
    let payment_id = String::from_str(&env, "pay_1");
    let event_id = String::from_str(&env, "event_1");
    let buyer = Address::generate(&env);

    let payment = Payment {
        payment_id: payment_id.clone(),
        event_id: event_id.clone(),
        buyer_address: buyer.clone(),
        ticket_tier_id: String::from_str(&env, "tier_1"),
        amount: 100000000,          // 100 USDC
        platform_fee: 2000000,      // 2 USDC
        organizer_amount: 98000000, // 98 USDC
        status: PaymentStatus::Pending,
        transaction_hash: String::from_str(&env, "hash_1"),
        created_at: 123456789,
        confirmed_at: None,
    };

    env.as_contract(&contract_id, || {
        store_payment(&env, payment.clone());
        let stored = get_payment(&env, payment_id).unwrap();
        assert_eq!(stored, payment);
    });
}

#[test]
fn test_update_payment_status() {
    let env = Env::default();
    let contract_id = env.register(TicketPaymentContract, ());
    let payment_id = String::from_str(&env, "pay_1");
    let event_id = String::from_str(&env, "event_1");
    let buyer = Address::generate(&env);

    let payment = Payment {
        payment_id: payment_id.clone(),
        event_id,
        buyer_address: buyer,
        ticket_tier_id: String::from_str(&env, "tier_1"),
        amount: 100,
        platform_fee: 2,
        organizer_amount: 98,
        status: PaymentStatus::Pending,
        transaction_hash: String::from_str(&env, "hash_1"),
        created_at: 123456789,
        confirmed_at: None,
    };

    env.as_contract(&contract_id, || {
        store_payment(&env, payment);
        update_payment_status(
            &env,
            payment_id.clone(),
            PaymentStatus::Confirmed,
            Some(123456790),
        );
        let updated = get_payment(&env, payment_id).unwrap();
        assert_eq!(updated.status, PaymentStatus::Confirmed);
        assert_eq!(updated.confirmed_at, Some(123456790));
    });
}

#[test]
fn test_indexing() {
    let env = Env::default();
    let contract_id = env.register(TicketPaymentContract, ());
    let event_id = String::from_str(&env, "event_1");
    let buyer = Address::generate(&env);

    let p1 = Payment {
        payment_id: String::from_str(&env, "pay_1"),
        event_id: event_id.clone(),
        buyer_address: buyer.clone(),
        ticket_tier_id: String::from_str(&env, "tier_1"),
        amount: 100,
        platform_fee: 2,
        organizer_amount: 98,
        status: PaymentStatus::Pending,
        transaction_hash: String::from_str(&env, "hash_1"),
        created_at: 123456789,
        confirmed_at: None,
    };

    let p2 = Payment {
        payment_id: String::from_str(&env, "pay_2"),
        event_id: event_id.clone(),
        buyer_address: buyer.clone(),
        ticket_tier_id: String::from_str(&env, "tier_2"),
        amount: 200,
        platform_fee: 4,
        organizer_amount: 196,
        status: PaymentStatus::Pending,
        transaction_hash: String::from_str(&env, "hash_2"),
        created_at: 123456790,
        confirmed_at: None,
    };

    env.as_contract(&contract_id, || {
        store_payment(&env, p1);
        store_payment(&env, p2);

        let event_payments = get_event_payments(&env, event_id);
        assert_eq!(event_payments.len(), 2);
        assert_eq!(
            event_payments.get(0).unwrap(),
            String::from_str(&env, "pay_1")
        );
        assert_eq!(
            event_payments.get(1).unwrap(),
            String::from_str(&env, "pay_2")
        );

        let buyer_payments = get_buyer_payments(&env, buyer);
        assert_eq!(buyer_payments.len(), 2);
    });
}

#[test]
fn test_config_storage() {
    let env = Env::default();
    let contract_id = env.register(TicketPaymentContract, ());
    let usdc = Address::generate(&env);
    let platform = Address::generate(&env);
    let registry = Address::generate(&env);

    env.as_contract(&contract_id, || {
        set_usdc_token(&env, usdc.clone());
        set_platform_wallet(&env, platform.clone());
        set_event_registry(&env, registry.clone());

        assert_eq!(get_usdc_token(&env), usdc);
        assert_eq!(get_platform_wallet(&env), platform);
        assert_eq!(get_event_registry(&env), registry);
    });
}
