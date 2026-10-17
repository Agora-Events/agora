//! Ticket counter and state management using the standard Soroban bumpable counter pattern.
//!
//! # Soroban Bumpable Counter Pattern
//! In Soroban's state archival model, every storage entry on the ledger carries a Time-To-Live (TTL).
//! When tickets are sold, updating a persistent counter requires reading the current value, applying
//! saturating arithmetic, saving the updated value, and extending the TTL of the storage entry.
//!
//! Extending the TTL (bumping) ensures that active platform data does not lapse into archival state,
//! avoiding expensive on-chain restoration fees and optimizing transaction gas costs.

#![no_std]

use soroban_sdk::{contracttype, Env, Symbol};

/// Storage key for the tickets sold counter.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TicketCounterKey {
    /// Global tickets sold counter across all events
    TicketsSold,
    /// Event-specific tickets sold counter
    EventTicketsSold(soroban_sdk::String),
}

/// Number of ledgers to extend the TTL when bumped (~30 days at ~5s per ledger).
pub const BUMP_AMOUNT: u32 = 518_400;

/// Lifetime threshold below which the entry TTL is bumped (~7 days at ~5s per ledger).
pub const LIFETIME_THRESHOLD: u32 = 120_960;

/// Returns the total number of tickets sold and extends its persistent storage TTL.
///
/// Calling this ensures the counter entry is kept active and warmed up in storage.
pub fn get_tickets_sold(env: &Env) -> i128 {
    let key = TicketCounterKey::TicketsSold;
    let count: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    env.storage().persistent().extend_ttl(&key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
    count
}

/// Increments the tickets sold counter by `quantity` and bumps the entry's TTL.
///
/// # Gas and Rent Optimization
/// 1. Reads the current count using `.get(&key).unwrap_or(0)`
/// 2. Performs safe saturating arithmetic `current.saturating_add(quantity)`
/// 3. Writes the updated value back to persistent storage
/// 4. Extends the TTL with `extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT)` so the entry
///    is guaranteed to live at least `BUMP_AMOUNT` ledgers into the future.
pub fn increment_tickets_sold(env: &Env, quantity: i128) -> i128 {
    let key = TicketCounterKey::TicketsSold;
    let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    let new_total = current.saturating_add(quantity);
    env.storage().persistent().set(&key, &new_total);
    env.storage().persistent().extend_ttl(&key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
    new_total
}

/// Decrements the tickets sold counter by `quantity` (e.g. on refund or cancellation)
/// and bumps the entry's TTL.
pub fn decrement_tickets_sold(env: &Env, quantity: i128) -> i128 {
    let key = TicketCounterKey::TicketsSold;
    let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    let new_total = current.saturating_sub(quantity);
    env.storage().persistent().set(&key, &new_total);
    env.storage().persistent().extend_ttl(&key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
    new_total
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_counter_lifecycle() {
        let env = Env::default();
        assert_eq!(get_tickets_sold(&env), 0);

        let sold = increment_tickets_sold(&env, 10);
        assert_eq!(sold, 10);
        assert_eq!(get_tickets_sold(&env), 10);

        let sold = decrement_tickets_sold(&env, 3);
        assert_eq!(sold, 7);
        assert_eq!(get_tickets_sold(&env), 7);
    }
}
