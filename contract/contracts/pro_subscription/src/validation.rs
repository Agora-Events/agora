use soroban_sdk::Env;

use crate::error::ProSubscriptionError;

/// Validates that an address is not the contract itself
pub fn validate_address(
    env: &Env,
    addr: &soroban_sdk::Address,
) -> Result<(), ProSubscriptionError> {
    if addr == &env.current_contract_address() {
        return Err(ProSubscriptionError::InvalidAddress);
    }
    Ok(())
}
