/// Unit tests for `get_global_promo` – issue #936
///
/// Covers three scenarios required by the acceptance criteria:
///   1. No promo set    → `get_global_promo` returns `None`
///   2. Active promo    → returns `Some((bps, expiry))`
///   3. Expired promo   → returns `None` after the ledger time passes expiry
///
/// The environment ledger timestamp is mocked via `env.ledger().set(...)` so
/// no real time passes and tests remain deterministic.
#[cfg(test)]
mod get_global_promo_tests {
    use crate::{EventRegistry, EventRegistryClient};
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        Address, Env,
    };

    // ── helpers ────────────────────────────────────────────────────────────

    /// Minimal ledger info used to seed the environment clock.
    fn ledger_at(timestamp: u64) -> LedgerInfo {
        LedgerInfo {
            timestamp,
            protocol_version: 23,
            sequence_number: 1,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 6_312_000,
        }
    }

    /// Register the contract and call `initialize`.
    fn setup(env: &Env) -> (EventRegistryClient<'static>, Address) {
        let contract_id = env.register(EventRegistry, ());
        let client = EventRegistryClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let platform_wallet = Address::generate(env);
        let usdc_token = Address::generate(env);

        client.initialize(&admin, &platform_wallet, &500, &usdc_token);
        (client, admin)
    }

    // ── test 1: no promo set ───────────────────────────────────────────────

    /// When no promo has ever been configured, `get_global_promo` must return
    /// `None`.
    #[test]
    fn test_get_global_promo_returns_none_when_not_set() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set(ledger_at(1_000));

        let (client, _admin) = setup(&env);

        // No `set_global_promo` call → should return None.
        let result = client.get_global_promo();
        assert!(
            result.is_none(),
            "expected None when no promo has been configured, got {:?}",
            result
        );
    }

    // ── test 2: active promo ───────────────────────────────────────────────

    /// When a promo is set with an expiry in the future, `get_global_promo`
    /// must return `Some((bps, expiry))` with the correct values.
    #[test]
    fn test_get_global_promo_returns_some_when_active() {
        let env = Env::default();
        env.mock_all_auths();

        let now: u64 = 1_000_000;
        env.ledger().set(ledger_at(now));

        let (client, _admin) = setup(&env);

        let discount_bps: u32 = 1_500; // 15%
        let expiry: u64 = now + 86_400; // expires in 24 h

        client.set_global_promo(&discount_bps, &expiry);

        let result = client.get_global_promo();
        assert!(
            result.is_some(),
            "expected Some for an active promo, got None"
        );

        let (returned_bps, returned_expiry) = result.unwrap();
        assert_eq!(
            returned_bps, discount_bps,
            "BPS mismatch: expected {discount_bps}, got {returned_bps}"
        );
        assert_eq!(
            returned_expiry, expiry,
            "expiry mismatch: expected {expiry}, got {returned_expiry}"
        );
    }

    // ── test 3: expired promo ──────────────────────────────────────────────

    /// After the ledger timestamp advances past the promo expiry, `get_global_promo`
    /// must return `None` – even though the promo data is still in storage.
    #[test]
    fn test_get_global_promo_returns_none_after_expiry() {
        let env = Env::default();
        env.mock_all_auths();

        let set_time: u64 = 1_000_000;
        env.ledger().set(ledger_at(set_time));

        let (client, _admin) = setup(&env);

        let discount_bps: u32 = 500; // 5%
        let expiry: u64 = set_time + 3_600; // expires in 1 h

        client.set_global_promo(&discount_bps, &expiry);

        // Verify promo is active before expiry.
        assert!(
            client.get_global_promo().is_some(),
            "promo should be active before expiry"
        );

        // Advance the ledger past the expiry timestamp.
        let after_expiry: u64 = expiry + 1;
        env.ledger().set(ledger_at(after_expiry));

        let result = client.get_global_promo();
        assert!(
            result.is_none(),
            "expected None after promo expiry, got {:?}",
            result
        );
    }
}

/// Unit tests for rejecting already-expired promos – issue #1738.
#[cfg(test)]
mod set_global_promo_expiry_validation_tests {
    use crate::{error::EventRegistryError, EventRegistry, EventRegistryClient};
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        Address, Env,
    };

    fn setup(env: &Env, now: u64) -> EventRegistryClient<'static> {
        env.ledger().set(LedgerInfo {
            timestamp: now,
            protocol_version: 23,
            sequence_number: 1,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 6_312_000,
        });
        let contract_id = env.register(EventRegistry, ());
        let client = EventRegistryClient::new(env, &contract_id);
        client.initialize(
            &Address::generate(env),
            &Address::generate(env),
            &500,
            &Address::generate(env),
        );
        client
    }

    #[test]
    fn test_past_expiry_with_discount_is_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let client = setup(&env, 1_000);

        let result = client.try_set_global_promo(&500, &999);
        assert_eq!(result, Err(Ok(EventRegistryError::InvalidDeadline)));
        assert_eq!(client.get_global_promo(), None);
    }

    #[test]
    fn test_expiry_equal_to_now_is_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let client = setup(&env, 1_000);

        let result = client.try_set_global_promo(&500, &1_000);
        assert_eq!(result, Err(Ok(EventRegistryError::InvalidDeadline)));
    }

    #[test]
    fn test_future_expiry_is_accepted() {
        let env = Env::default();
        env.mock_all_auths();
        let client = setup(&env, 1_000);

        client.set_global_promo(&500, &2_000);
        assert_eq!(client.get_global_promo(), Some((500, 2_000)));
    }

    #[test]
    fn test_clearing_with_zero_bps_accepts_any_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        let client = setup(&env, 1_000);

        client.set_global_promo(&500, &2_000);
        // Past expiry is fine when the call only clears the promo.
        client.set_global_promo(&0, &1);
        assert_eq!(client.get_global_promo_bps(), 0);
        assert_eq!(client.get_global_promo(), None);
    }
}
