use soroban_sdk::{Env, Symbol, symbol_short};

use crate::location::{validate_location, Location};

const EVENT_LOCATION: Symbol = symbol_short!("EVENT_LOC");

pub fn set_event_location(env: &Env, event_id: u64, location: Location) {
    validate_location(location.lat, location.long).unwrap();

    env.storage()
        .persistent()
        .set(&(EVENT_LOCATION, event_id), &location);
}

pub fn get_event_location(env: &Env, event_id: u64) -> Option<Location> {
    env.storage().persistent().get(&(EVENT_LOCATION, event_id))
}
