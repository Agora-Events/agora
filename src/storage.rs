use soroban_sdk::{Env, Symbol, IntoVal};

use crate::location::{Location, validate_location};

const EVENT_LOCATION: Symbol = Symbol::short("EVENT_LOC");

pub fn set_event_location(env: &Env, event_id: u64, location: Location) {
    validate_location(location.lat, location.long).unwrap();

    env.storage()
        .persistent()
        .set(&(EVENT_LOCATION, event_id), location);
}

pub fn get_event_location(env: &Env, event_id: u64) -> Option<Location> {
    env.storage()
        .persistent()
        .get(&(EVENT_LOCATION, event_id))
}
