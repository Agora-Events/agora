use soroban_sdk::{contracterror};

/// Custom error types for the Ticket Payment contract
#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Error {
    InvalidAmount = 1,
    InsufficientBalance = 2,
    PaymentNotFound = 3,
    PaymentAlreadyConfirmed = 4,
    InvalidEventId = 5,
    EventRegistryError = 6,
    TransferFailed = 7,
    Overflow = 8,
    Unauthorized = 9,
}
