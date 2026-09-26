use crate::{
    error::EventRegistryError,
    storage,
    types::{Dispute, DisputeStatus, DisputeVote},
};
use soroban_sdk::{contractclient, Address, Env, String, Vec};

#[contractclient(name = "TicketPaymentClient")]
pub trait TicketPaymentInterface {
    fn has_confirmed_ticket(env: Env, event_id: String, address: Address) -> bool;
}

const DISPUTE_VOTING_DURATION: u64 = 172800;
const DEFAULT_QUORUM_BPS: u32 = 3000;

pub fn open_dispute(
    env: &Env,
    event_id: String,
    opened_by: Address,
) -> Result<(), EventRegistryError> {
    opened_by.require_auth();

    let event_info =
        storage::get_event(env, event_id.clone()).ok_or(EventRegistryError::EventNotFound)?;

    if storage::get_dispute(env, event_id.clone()).is_some() {
        return Err(EventRegistryError::EventAlreadyExists);
    }

    let now = env.ledger().timestamp();
    if event_info.end_time == 0 || now > event_info.end_time + 172800 {
        return Err(EventRegistryError::EventNotEnded);
    }

    let ticket_payment_addr =
        storage::get_ticket_payment_contract(env).ok_or(EventRegistryError::NotInitialized)?;
    let tp_client = TicketPaymentClient::new(env, &ticket_payment_addr);
    if !tp_client.has_confirmed_ticket(&event_id, &opened_by) {
        return Err(EventRegistryError::NotTicketHolder);
    }

    let closes_at = now + DISPUTE_VOTING_DURATION;

    let dispute = Dispute {
        event_id: event_id.clone(),
        opened_by: opened_by.clone(),
        opened_at: now,
        closes_at,
        status: DisputeStatus::Open,
        total_votes: 0,
        buyer_votes: 0,
        organizer_votes: 0,
        quorum_threshold_bps: DEFAULT_QUORUM_BPS,
        total_eligible_tickets: 0,
    };

    storage::store_dispute(env, &dispute);

    env.events().publish(
        (crate::events::AgoraEvent::DisputeOpened,),
        crate::events::DisputeOpenedEvent {
            event_id,
            opened_by,
            timestamp: now,
        },
    );

    Ok(())
}

pub fn vote_on_dispute(
    env: &Env,
    event_id: String,
    voter: Address,
    vote: DisputeVote,
) -> Result<(), EventRegistryError> {
    voter.require_auth();

    let mut dispute =
        storage::get_dispute(env, event_id.clone()).ok_or(EventRegistryError::DisputeNotFound)?;

    if dispute.status != DisputeStatus::Open && dispute.status != DisputeStatus::Voting {
        return Err(EventRegistryError::DisputeNotOpen);
    }

    if env.ledger().timestamp() > dispute.closes_at {
        return Err(EventRegistryError::ProposalExpired);
    }

    if storage::has_voted(env, event_id.clone(), &voter) {
        return Err(EventRegistryError::AlreadyVoted);
    }

    if dispute.status == DisputeStatus::Open {
        dispute.status = DisputeStatus::Voting;
    }

    dispute.total_votes = dispute
        .total_votes
        .checked_add(1)
        .ok_or(EventRegistryError::SupplyOverflow)?;
    match vote {
        DisputeVote::BuyerFavor => {
            dispute.buyer_votes = dispute
                .buyer_votes
                .checked_add(1)
                .ok_or(EventRegistryError::SupplyOverflow)?;
        }
        DisputeVote::OrganizerFavor => {
            dispute.organizer_votes = dispute
                .organizer_votes
                .checked_add(1)
                .ok_or(EventRegistryError::SupplyOverflow)?;
        }
    }

    storage::store_dispute_vote(env, event_id.clone(), &voter, &vote);
    storage::add_dispute_vote(env, event_id.clone(), &voter);
    storage::store_dispute(env, &dispute);

    env.events().publish(
        (crate::events::AgoraEvent::DisputeVoted,),
        crate::events::DisputeVotedEvent {
            event_id,
            voter,
            vote,
            timestamp: env.ledger().timestamp(),
        },
    );

    Ok(())
}

pub fn resolve_dispute(env: &Env, event_id: String) -> Result<DisputeStatus, EventRegistryError> {
    let mut dispute =
        storage::get_dispute(env, event_id.clone()).ok_or(EventRegistryError::DisputeNotFound)?;

    if env.ledger().timestamp() <= dispute.closes_at {
        return Err(EventRegistryError::ProposalExpired);
    }

    if dispute.status == DisputeStatus::ResolvedBuyer
        || dispute.status == DisputeStatus::ResolvedOrganizer
    {
        return Ok(dispute.status);
    }

    let total = dispute.total_votes;
    let buyer_bps = dispute
        .buyer_votes
        .checked_mul(10000)
        .ok_or(EventRegistryError::SupplyOverflow)?;
    let meets_quorum = if total > 0 {
        buyer_bps / total as u32 >= dispute.quorum_threshold_bps
    } else {
        false
    };

    if meets_quorum && dispute.buyer_votes > dispute.organizer_votes {
        dispute.status = DisputeStatus::ResolvedBuyer;
    } else {
        dispute.status = DisputeStatus::ResolvedOrganizer;
    }

    storage::store_dispute(env, &dispute);

    env.events().publish(
        (crate::events::AgoraEvent::DisputeResolved,),
        crate::events::DisputeResolvedEvent {
            event_id,
            status: dispute.status.clone(),
            buyer_votes: dispute.buyer_votes,
            organizer_votes: dispute.organizer_votes,
            total_votes: dispute.total_votes,
            timestamp: env.ledger().timestamp(),
        },
    );

    Ok(dispute.status)
}

pub fn get_dispute(env: &Env, event_id: String) -> Option<Dispute> {
    storage::get_dispute(env, event_id)
}

pub fn get_dispute_votes(env: &Env, event_id: String) -> Vec<DisputeVote> {
    storage::get_dispute_votes(env, event_id)
}
