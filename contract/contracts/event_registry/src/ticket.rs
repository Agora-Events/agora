//! Ticket counter and state management using the standard Soroban bumpable counter pattern.
//!
//! # Soroban Bumpable Counter Pattern
//! In Soroban's state archival and rent model, persistent storage entries expire if their
//! Time-To-Live (TTL) is not periodically extended. If an entry expires, it is archived and
//! restoring it requires paying higher gas and restoration fees.
//!
//! To optimize gas consumption and ensure reliable state access:
//! 1. Counter values are read from persistent storage with default zero fallback.
//! 2. Counter mutations use saturating arithmetic to prevent overflows.
//! 3. After mutating or accessing the counter, [`bump_persistent`] extends the entry's TTL
//!    to [`PERSISTENT_BUMP_AMOUNT`] ledgers whenever it falls below [`PERSISTENT_LIFETIME_THRESHOLD`].
//!    This prevents unnecessary ledger writes when the entry is already fresh.

use crate::storage::{bump_persistent, get_global_tickets_sold};
use crate::types::DataKey;
use soroban_sdk::Env;

/// Returns the total number of tickets sold across all events and bumps the counter TTL.
///
/// Bumping the counter's persistent storage key on read keeps the counter active
/// for future transactions, avoiding cold-load / archival costs.
pub fn get_tickets_sold(env: &Env) -> i128 {
    get_global_tickets_sold(env)
}

/// Increments the tickets sold counter by `quantity` using the standard Soroban
/// bumpable counter pattern and extends the entry's TTL.
///
/// # Gas Optimization
/// Bumping the entry TTL whenever tickets are sold prevents entry expiration and eliminates
/// the need for costly state revival transactions. The TTL is only extended if it falls
/// below [`crate::storage::PERSISTENT_LIFETIME_THRESHOLD`], minimizing unnecessary state updates.
pub fn increment_tickets_sold(env: &Env, quantity: i128) -> i128 {
    let key = DataKey::GlobalTicketsSold;
    let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    let new_total = current.saturating_add(quantity);
    env.storage().persistent().set(&key, &new_total);
    bump_persistent(env, &key);
    new_total
}

/// Decrements the tickets sold counter by `quantity` using the standard Soroban
/// bumpable counter pattern and extends the entry's TTL.
pub fn decrement_tickets_sold(env: &Env, quantity: i128) -> i128 {
    let key = DataKey::GlobalTicketsSold;
    let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    let new_total = current.saturating_sub(quantity);
    env.storage().persistent().set(&key, &new_total);
    bump_persistent(env, &key);
    new_total
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_ticket_counter_lifecycle() {
        let env = Env::default();
        assert_eq!(get_tickets_sold(&env), 0);

        let sold = increment_tickets_sold(&env, 5);
        assert_eq!(sold, 5);
        assert_eq!(get_tickets_sold(&env), 5);

        let sold = increment_tickets_sold(&env, 3);
        assert_eq!(sold, 8);
        assert_eq!(get_tickets_sold(&env), 8);

        let sold = decrement_tickets_sold(&env, 2);
        assert_eq!(sold, 6);
        assert_eq!(get_tickets_sold(&env), 6);
    }
}
