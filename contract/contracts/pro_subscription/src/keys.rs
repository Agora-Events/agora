use soroban_sdk::contracttype;

/// Storage keys for the pro_subscription contract.
/// All persistent and instance storage lookups use these variants.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Subscription record keyed by subscriber address
    Subscription(soroban_sdk::Address),
    /// Admin address allowed to manage subscriptions
    Admin,
    /// Global subscription tier configuration
    TierConfig,
}
