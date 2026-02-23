#![no_std]

use crate::events::{
    AgoraEvent, EventRegisteredEvent, EventStatusUpdatedEvent, FeeUpdatedEvent,
    InitializationEvent, InventoryIncrementedEvent, MetadataUpdatedEvent, RegistryUpgradedEvent,
};
use crate::types::{EventInfo, PaymentInfo, TicketTier};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, String, Vec};

pub mod error;
pub mod events;
pub mod storage;
pub mod types;

use crate::error::EventRegistryError;

#[contract]
pub struct EventRegistry;

#[contractimpl]
#[allow(deprecated)]
impl EventRegistry {
    /// Initializes the contract configuration. Can only be called once.
    ///
    /// # Arguments
    /// * `admin` - The administrator address.
    /// * `platform_wallet` - The platform wallet address for fees.
    /// * `platform_fee_percent` - Initial platform fee in basis points (10000 = 100%).
    pub fn initialize(
        env: Env,
        admin: Address,
        platform_wallet: Address,
        platform_fee_percent: u32,
    ) -> Result<(), EventRegistryError> {
        if storage::is_initialized(&env) {
            return Err(EventRegistryError::AlreadyInitialized);
        }

        validate_address(&env, &admin)?;
        validate_address(&env, &platform_wallet)?;

        let initial_fee = if platform_fee_percent == 0 {
            500
        } else {
            platform_fee_percent
        };

        if initial_fee > 10000 {
            return Err(EventRegistryError::InvalidFeePercent);
        }
        storage::set_admin(&env, &admin);
        storage::set_platform_wallet(&env, &platform_wallet);
        storage::set_platform_fee(&env, initial_fee);
        storage::set_initialized(&env, true);

        env.events().publish(
            (AgoraEvent::ContractInitialized,),
            InitializationEvent {
                admin_address: admin,
                platform_wallet,
                platform_fee_percent: initial_fee,
                timestamp: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    /// Register a new event with organizer authentication and tiered pricing
    ///
    /// # Arguments
    /// * `event_id` - Unique identifier for the event
    /// * `organizer_address` - The wallet address of the event organizer
    /// * `payment_address` - The address where payments should be routed
    /// * `metadata_cid` - IPFS CID for event metadata
    /// * `max_supply` - Maximum number of tickets (0 = unlimited)
    /// * `tiers` - Map of tier_id to TicketTier for multi-tiered pricing
    pub fn register_event(
        env: Env,
        event_id: String,
        organizer_address: Address,
        payment_address: Address,
        metadata_cid: String,
        max_supply: i128,
        tiers: Map<String, TicketTier>,
    ) -> Result<(), EventRegistryError> {
        if !storage::is_initialized(&env) {
            return Err(EventRegistryError::NotInitialized);
        }
        organizer_address.require_auth();

        validate_metadata_cid(&env, &metadata_cid)?;

        if storage::event_exists(&env, event_id.clone()) {
            return Err(EventRegistryError::EventAlreadyExists);
        }

        // Validate tier limits don't exceed max_supply
        if max_supply > 0 {
            let mut total_tier_limit: i128 = 0;
            for tier in tiers.values() {
                total_tier_limit = total_tier_limit
                    .checked_add(tier.tier_limit)
                    .ok_or(EventRegistryError::SupplyOverflow)?;
            }
            if total_tier_limit > max_supply {
                return Err(EventRegistryError::TierLimitExceedsMaxSupply);
            }
        }

        let platform_fee_percent = storage::get_platform_fee(&env);

        let event_info = EventInfo {
            event_id: event_id.clone(),
            organizer_address: organizer_address.clone(),
            payment_address: payment_address.clone(),
            platform_fee_percent,
            is_active: true,
            created_at: env.ledger().timestamp(),
            metadata_cid,
            max_supply,
            current_supply: 0,
            tiers,
            custom_fee_bps: None,
        };

        storage::store_event(&env, event_info);

        env.events().publish(
            (AgoraEvent::EventRegistered,),
            EventRegisteredEvent {
                event_id: event_id.clone(),
                organizer_address: organizer_address.clone(),
                payment_address: payment_address.clone(),
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Get event payment information including tiered pricing
    pub fn get_event_payment_info(
        env: Env,
        event_id: String,
    ) -> Result<PaymentInfo, EventRegistryError> {
        match storage::get_event(&env, event_id) {
            Some(event_info) => {
                if !event_info.is_active {
                    return Err(EventRegistryError::EventInactive);
                }
                let effective_fee = event_info.custom_fee_bps.unwrap_or(event_info.platform_fee_percent);
                Ok(PaymentInfo {
                    payment_address: event_info.payment_address,
                    platform_fee_percent: effective_fee,
                    tiers: event_info.tiers,
                })
            }
            None => Err(EventRegistryError::EventNotFound),
        }
    }

    /// Update event status (only by organizer)
    pub fn update_event_status(
        env: Env,
        event_id: String,
        is_active: bool,
    ) -> Result<(), EventRegistryError> {
        match storage::get_event(&env, event_id.clone()) {
            Some(mut event_info) => {
                // Verify organizer signature
                event_info.organizer_address.require_auth();

                // Update status
                event_info.is_active = is_active;
                storage::store_event(&env, event_info.clone());

                // Emit status update event using contract event type
                env.events().publish(
                    (AgoraEvent::EventStatusUpdated,),
                    EventStatusUpdatedEvent {
                        event_id,
                        is_active,
                        updated_by: event_info.organizer_address,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                Ok(())
            }
            None => Err(EventRegistryError::EventNotFound),
        }
    }

    /// Update the decentralized metadata CID for an event (only by organizer)
    pub fn update_metadata(
        env: Env,
        event_id: String,
        new_metadata_cid: String,
    ) -> Result<(), EventRegistryError> {
        match storage::get_event(&env, event_id.clone()) {
            Some(mut event_info) => {
                // Verify organizer signature
                event_info.organizer_address.require_auth();

                // Validate new metadata CID
                validate_metadata_cid(&env, &new_metadata_cid)?;

                // Update metadata
                event_info.metadata_cid = new_metadata_cid.clone();
                storage::store_event(&env, event_info.clone());

                // Emit metadata update event
                env.events().publish(
                    (AgoraEvent::MetadataUpdated,),
                    MetadataUpdatedEvent {
                        event_id,
                        new_metadata_cid,
                        updated_by: event_info.organizer_address,
                        timestamp: env.ledger().timestamp(),
                    },
                );

                Ok(())
            }
            None => Err(EventRegistryError::EventNotFound),
        }
    }

    /// Stores or updates an event (legacy function for backward compatibility).
    pub fn store_event(env: Env, event_info: EventInfo) {
        // In a real scenario, we would check authorization here.
        storage::store_event(&env, event_info);
    }

    /// Retrieves an event by its ID.
    pub fn get_event(env: Env, event_id: String) -> Option<EventInfo> {
        storage::get_event(&env, event_id)
    }

    /// Checks if an event exists.
    pub fn event_exists(env: Env, event_id: String) -> bool {
        storage::event_exists(&env, event_id)
    }

    /// Retrieves all event IDs for an organizer.
    pub fn get_organizer_events(env: Env, organizer: Address) -> Vec<String> {
        storage::get_organizer_events(&env, &organizer)
    }

    /// Updates the platform fee percentage. Only callable by the administrator.
    pub fn set_platform_fee(env: Env, new_fee_percent: u32) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        if new_fee_percent > 10000 {
            return Err(EventRegistryError::InvalidFeePercent);
        }

        storage::set_platform_fee(&env, new_fee_percent);

        // Emit fee update event using contract event type
        env.events().publish(
            (AgoraEvent::FeeUpdated,),
            FeeUpdatedEvent { new_fee_percent },
        );

        Ok(())
    }

    /// Returns the current platform fee percentage.
    pub fn get_platform_fee(env: Env) -> u32 {
        storage::get_platform_fee(&env)
    }

    /// Returns the current administrator address.
    pub fn get_admin(env: Env) -> Result<Address, EventRegistryError> {
        storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)
    }

    /// Returns the current platform wallet address.
    pub fn get_platform_wallet(env: Env) -> Result<Address, EventRegistryError> {
        storage::get_platform_wallet(&env).ok_or(EventRegistryError::NotInitialized)
    }

    /// Sets the authorized TicketPayment contract address. Only callable by the administrator.
    ///
    /// # Arguments
    /// * `ticket_payment_address` - The address of the TicketPayment contract authorized
    ///   to call `increment_inventory`.
    pub fn set_ticket_payment_contract(
        env: Env,
        ticket_payment_address: Address,
    ) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        validate_address(&env, &ticket_payment_address)?;

        storage::set_ticket_payment_contract(&env, &ticket_payment_address);
        Ok(())
    }

    /// Sets a custom fee override for a specific event. Only callable by the administrator.
    ///
    /// # Arguments
    /// * `event_id` - The event to apply the custom fee to
    /// * `custom_fee_bps` - Custom fee in basis points (10000 = 100%), or None to remove override
    pub fn set_custom_event_fee(
        env: Env,
        event_id: String,
        custom_fee_bps: Option<u32>,
    ) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        if let Some(fee) = custom_fee_bps {
            if fee > 10000 {
                return Err(EventRegistryError::InvalidFeePercent);
            }
        }

        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        event_info.custom_fee_bps = custom_fee_bps;
        storage::store_event(&env, event_info);

        Ok(())
    }

    /// Returns the authorized TicketPayment contract address.
    pub fn get_ticket_payment_contract(env: Env) -> Result<Address, EventRegistryError> {
        storage::get_ticket_payment_contract(&env).ok_or(EventRegistryError::NotInitialized)
    }

    /// Increments the current_supply counter for a given event and tier.
    /// This function is restricted to calls from the authorized TicketPayment contract.
    ///
    /// # Arguments
    /// * `event_id` - The event whose inventory to increment.
    /// * `tier_id` - The tier whose inventory to increment.
    ///
    /// # Errors
    /// * `UnauthorizedCaller` - If the invoker is not the registered TicketPayment contract.
    /// * `EventNotFound` - If no event with the given ID exists.
    /// * `EventInactive` - If the event is not currently active.
    /// * `TierNotFound` - If the tier does not exist.
    /// * `TierSupplyExceeded` - If the tier's limit has been reached.
    /// * `MaxSupplyExceeded` - If the event's max supply has been reached (when max_supply > 0).
    /// * `SupplyOverflow` - If incrementing would cause an i128 overflow.
    pub fn increment_inventory(
        env: Env,
        event_id: String,
        tier_id: String,
    ) -> Result<(), EventRegistryError> {
        let ticket_payment_addr =
            storage::get_ticket_payment_contract(&env).ok_or(EventRegistryError::NotInitialized)?;
        ticket_payment_addr.require_auth();

        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        if !event_info.is_active {
            return Err(EventRegistryError::EventInactive);
        }

        // Check global supply limits
        if event_info.max_supply > 0 && event_info.current_supply >= event_info.max_supply {
            return Err(EventRegistryError::MaxSupplyExceeded);
        }

        // Get and update tier
        let mut tier = event_info
            .tiers
            .get(tier_id.clone())
            .ok_or(EventRegistryError::TierNotFound)?;

        if tier.current_sold >= tier.tier_limit {
            return Err(EventRegistryError::TierSupplyExceeded);
        }

        tier.current_sold = tier
            .current_sold
            .checked_add(1)
            .ok_or(EventRegistryError::SupplyOverflow)?;

        event_info.tiers.set(tier_id, tier);

        event_info.current_supply = event_info
            .current_supply
            .checked_add(1)
            .ok_or(EventRegistryError::SupplyOverflow)?;

        storage::store_event(&env, event_info.clone());

        env.events().publish(
            (AgoraEvent::InventoryIncremented,),
            InventoryIncrementedEvent {
                event_id,
                new_supply: event_info.current_supply,
                max_supply: event_info.max_supply,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Decrements the current_supply counter for a given event and tier.
    /// This function is restricted to calls from the authorized TicketPayment contract upon refund.
    ///
    /// # Arguments
    /// * `event_id` - The event whose inventory to decrement.
    /// * `tier_id` - The tier whose inventory to decrement.
    ///
    /// # Errors
    /// * `UnauthorizedCaller` - If the invoker is not the registered TicketPayment contract.
    /// * `EventNotFound` - If no event with the given ID exists.
    /// * `TierNotFound` - If the tier does not exist.
    /// * `SupplyUnderflow` - If decrementing would cause the supply to go below 0.
    pub fn decrement_inventory(
        env: Env,
        event_id: String,
        tier_id: String,
    ) -> Result<(), EventRegistryError> {
        let ticket_payment_addr =
            storage::get_ticket_payment_contract(&env).ok_or(EventRegistryError::NotInitialized)?;
        ticket_payment_addr.require_auth();

        let mut event_info =
            storage::get_event(&env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

        // Get and update tier
        let mut tier = event_info
            .tiers
            .get(tier_id.clone())
            .ok_or(EventRegistryError::TierNotFound)?;

        if tier.current_sold <= 0 {
            return Err(EventRegistryError::SupplyUnderflow);
        }

        tier.current_sold = tier
            .current_sold
            .checked_sub(1)
            .ok_or(EventRegistryError::SupplyUnderflow)?;

        event_info.tiers.set(tier_id, tier);

        if event_info.current_supply <= 0 {
            return Err(EventRegistryError::SupplyUnderflow);
        }

        event_info.current_supply = event_info
            .current_supply
            .checked_sub(1)
            .ok_or(EventRegistryError::SupplyUnderflow)?;

        storage::store_event(&env, event_info.clone());

        env.events().publish(
            (crate::events::AgoraEvent::InventoryDecremented,),
            crate::events::InventoryDecrementedEvent {
                event_id,
                new_supply: event_info.current_supply,
                max_supply: event_info.max_supply,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    /// Upgrades the contract to a new WASM hash. Only callable by the administrator.
    /// Performs post-upgrade state verification to ensure critical storage is intact.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), EventRegistryError> {
        let admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        // Post-upgrade state verification
        let verified_admin = storage::get_admin(&env).ok_or(EventRegistryError::NotInitialized)?;
        storage::get_platform_wallet(&env).ok_or(EventRegistryError::NotInitialized)?;

        env.events().publish(
            (AgoraEvent::ContractUpgraded,),
            RegistryUpgradedEvent {
                admin_address: verified_admin,
                timestamp: env.ledger().timestamp(),
            },
        );

        Ok(())
    }
}

fn validate_address(env: &Env, address: &Address) -> Result<(), EventRegistryError> {
    if address == &env.current_contract_address() {
        return Err(EventRegistryError::InvalidAddress);
    }
    Ok(())
}

fn validate_metadata_cid(env: &Env, cid: &String) -> Result<(), EventRegistryError> {
    if cid.len() < 46 {
        return Err(EventRegistryError::InvalidMetadataCid);
    }

    // We expect CIDv1 base32, which starts with 'b'
    // Convert to Bytes to check the first character safely
    let mut bytes = soroban_sdk::Bytes::new(env);
    bytes.append(&cid.clone().into());

    if !bytes.is_empty() && bytes.get(0) != Some(b'b') {
        return Err(EventRegistryError::InvalidMetadataCid);
    }

    Ok(())
}

#[cfg(test)]
mod test;
