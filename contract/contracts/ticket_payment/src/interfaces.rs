// Price Oracle interface
pub mod price_oracle {
    use soroban_sdk::{contractclient, Address, Env};

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct PriceData {
        pub price: i128,
        pub timestamp: u64,
    }

    #[contractclient(name = "OracleClient")]
    pub trait PriceOracleInterface {
        fn lastprice(env: Env, asset: Address) -> Option<PriceData>;
    }
}

// Pro Subscription interface
pub mod pro_subscription {
    use soroban_sdk::{contractclient, Address, Env};

    #[contractclient(name = "ProSubscriptionClient")]
    pub trait ProSubscriptionInterface {
        fn is_pro_member(env: Env, organizer: Address) -> bool;
    }
}

// Event Registry interface
pub mod event_registry {
    use soroban_sdk::{contractclient, Address, Env, String};

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum EventStatus {
        Active,
        Inactive,
        Cancelled,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct PaymentInfo {
        pub payment_address: Address,
        pub platform_fee_percent: u32,
        pub custom_fee_bps: Option<u32>,
        pub referral_rate_bps: u32,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct EventInventory {
        pub current_supply: i128,
        pub max_supply: i128,
    }

    /// Loyalty profile mirrored from the event_registry contract
    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct GuestProfile {
        pub guest_address: Address,
        pub loyalty_score: u64,
        pub total_tickets_purchased: u32,
        pub total_spent: i128,
        pub last_updated: u64,
    }

    #[contractclient(name = "Client")]
    pub trait EventRegistryInterface {
        fn get_event_payment_info(env: Env, event_id: String) -> PaymentInfo;
        fn get_event(env: Env, event_id: String) -> Option<EventInfo>;
        fn get_organizer_address(env: Env, event_id: String) -> Option<Address>;
        fn increment_inventory(
            env: Env,
            event_id: String,
            tier_id: String,
            user: Address,
            quantity: u32,
        );
        fn decrement_inventory(env: Env, event_id: String, tier_id: String, user: Address);
        fn get_global_promo_bps(env: Env) -> u32;
        fn get_promo_expiry(env: Env) -> u64;
        fn is_scanner_authorized(env: Env, event_id: String, scanner: Address) -> bool;
        fn update_loyalty_score(
            env: Env,
            caller: Address,
            guest: Address,
            tickets_purchased: u32,
            amount_spent: i128,
            loyalty_multiplier: u32,
        );
        fn get_loyalty_discount_bps(env: Env, guest: Address) -> u32;
        fn get_guest_profile(env: Env, guest: Address) -> Option<GuestProfile>;
    }

    pub use crate::types::AuctionConfig;

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct TicketTier {
        pub name: String,
        pub price: i128,
        pub early_bird_price: i128,
        pub early_bird_deadline: u64,
        pub usd_price: i128,
        pub tier_limit: i128,
        pub current_sold: i128,
        pub is_refundable: bool,
        pub auction_config: soroban_sdk::Vec<AuctionConfig>,
        pub loyalty_multiplier: u32,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Milestone {
        pub sales_threshold: i128,
        pub release_percent: u32,
    }

    #[soroban_sdk::contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct EventInfo {
        pub event_id: String,
        pub name: String,
        pub organizer_address: Address,
        pub payment_address: Address,
        pub platform_fee_percent: u32,
        pub is_active: bool,
        pub status: EventStatus,
        pub created_at: u64,
        pub metadata_cid: String,
        pub max_supply: i128,
        pub current_supply: i128,
        pub milestone_plan: Option<soroban_sdk::Vec<Milestone>>,
        pub tiers: soroban_sdk::Map<String, TicketTier>,
        pub refund_deadline: u64,
        pub restocking_fee: i128,
        pub resale_cap_bps: Option<u32>,
        pub min_sales_target: i128,
        pub target_deadline: u64,
        pub goal_met: bool,
        pub custom_fee_bps: Option<u32>,
        pub banner_cid: Option<String>,
        pub tags: Option<soroban_sdk::Vec<String>>,
        pub start_time: u64,
        pub end_time: u64,
        pub accepted_tokens: soroban_sdk::Vec<Address>,
        pub use_global_whitelist: bool,
        pub referral_rate_bps: u32,
    }
}
