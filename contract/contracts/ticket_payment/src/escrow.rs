use crate::{
    error::TicketPaymentError,
    interfaces::event_registry,
    storage::{get_event_balance, set_event_balance},
    types::DataKey,
};
use soroban_sdk::{contracttype, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowMilestone {
    pub sales_threshold: i128,
    pub release_percent: u32,
    pub released: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowState {
    pub total_collected: i128,
    pub total_released: i128,
    pub milestones_reached: u32,
}

pub fn release_escrow_milestone(
    env: &Env,
    event_id: String,
    _token_address: Address,
) -> Result<i128, TicketPaymentError> {
    let event_registry_addr = crate::storage::get_event_registry(env);
    let registry_client = event_registry::Client::new(env, &event_registry_addr);
    let event_info = registry_client
        .try_get_event(&event_id)
        .ok()
        .and_then(|r| r.ok())
        .flatten()
        .ok_or(TicketPaymentError::EventNotFound)?;

    if event_info.is_active || matches!(event_info.status, event_registry::EventStatus::Active) {
        return Err(TicketPaymentError::EventInactive);
    }

    event_info.organizer_address.require_auth();

    let mut escrow_state = get_escrow_state(env, event_id.clone())
        .ok_or(TicketPaymentError::EscrowNotInitialized)?;

    let milestones = get_escrow_milestones(env, event_id.clone());

    let mut amount_released = 0i128;
    let mut newly_reached = 0u32;

    for i in 0..milestones.len() {
        if (i as u32) < escrow_state.milestones_reached {
            continue;
        }
        let milestone = milestones.get(i).unwrap();
        if !milestone.released && event_info.current_supply >= milestone.sales_threshold {
            let balance = get_event_balance(env, event_id.clone());
            let release_amount = balance
                .organizer_amount
                .checked_mul(milestone.release_percent as i128)
                .and_then(|v| v.checked_div(10000))
                .ok_or(TicketPaymentError::ArithmeticError)?;

            if release_amount > 0 {
                amount_released = amount_released
                    .checked_add(release_amount)
                    .ok_or(TicketPaymentError::ArithmeticError)?;

                escrow_state.total_released = escrow_state
                    .total_released
                    .checked_add(release_amount)
                    .ok_or(TicketPaymentError::ArithmeticError)?;
            }

            newly_reached = newly_reached
                .checked_add(1)
                .ok_or(TicketPaymentError::ArithmeticError)?;
        }
    }

    if amount_released == 0 {
        return Ok(0);
    }

    escrow_state.milestones_reached = escrow_state
        .milestones_reached
        .checked_add(newly_reached)
        .ok_or(TicketPaymentError::ArithmeticError)?;

    let mut new_balance = get_event_balance(env, event_id.clone());
    new_balance.organizer_amount = new_balance
        .organizer_amount
        .checked_sub(amount_released)
        .ok_or(TicketPaymentError::ArithmeticError)?;
    new_balance.total_withdrawn = new_balance
        .total_withdrawn
        .checked_add(amount_released)
        .ok_or(TicketPaymentError::ArithmeticError)?;

    set_event_balance(env, event_id.clone(), new_balance);

    env.storage().persistent().set(
        &DataKey::EscrowState(event_id.clone()),
        &escrow_state,
    );

    #[allow(deprecated)]
    env.events().publish(
        (crate::events::AgoraEvent::MilestoneReleased,),
        crate::events::MilestoneReleasedEvent {
            event_id,
            milestone_index: escrow_state.milestones_reached,
            amount_released,
            timestamp: env.ledger().timestamp(),
        },
    );

    Ok(amount_released)
}

pub fn get_escrow_state(env: &Env, event_id: String) -> Option<EscrowState> {
    env.storage()
        .persistent()
        .get(&DataKey::EscrowState(event_id))
}

fn get_escrow_milestones(env: &Env, event_id: String) -> Vec<EscrowMilestone> {
    let mut result = Vec::new(env);
    for i in 0..u32::MAX {
        let key = DataKey::EscrowMilestone(event_id.clone(), i);
        if env.storage().persistent().has(&key) {
            let milestone: EscrowMilestone = env
                .storage()
                .persistent()
                .get(&key)
                .unwrap_or_else(|| EscrowMilestone {
                    sales_threshold: 0,
                    release_percent: 0,
                    released: false,
                });
            result.push_back(milestone);
        } else if i > 0 && result.is_empty() {
            break;
        } else if i > 0 {
            break;
        }
    }
    result
}

pub fn init_escrow_for_event(env: &Env, event_id: String) -> EscrowState {
    let state = EscrowState {
        total_collected: 0,
        total_released: 0,
        milestones_reached: 0,
    };
    env.storage()
        .persistent()
        .set(&DataKey::EscrowState(event_id.clone()), &state);
    state
}
