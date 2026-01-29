// use soroban_sdk::{contractevent, Address, String};

// Event emitted when a payment is successfully processed
// #[contractevent]
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct PaymentProcessed {
//     pub payment_id: String,
//     pub event_id: String,
//     pub buyer_address: Address,
//     pub amount: i128,
//     pub platform_fee: i128,
//     pub organizer_amount: i128,
//     pub timestamp: u64,
// }

// Event emitted when a payment is confirmed on the blockchain
// #[contractevent]
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct PaymentConfirmed {
//     pub payment_id: String,
//     pub transaction_hash: String,
//     pub confirmed_at: u64,
// }

// Event emitted when a payment fails
// #[contractevent]
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct PaymentFailed {
//     pub payment_id: String,
//     pub reason: String,
//     pub timestamp: u64,
// }
