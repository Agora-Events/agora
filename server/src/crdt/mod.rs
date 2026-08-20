/**
 * CRDT module for server-side conflict resolution
 * Provides Vector Clock, LWW-Element-Set, and OR-Set implementations
 */
pub mod vector_clock;
pub mod lww_element_set;
pub mod or_set;

#[cfg(test)]
mod tests;

pub use vector_clock::{VectorClock, VectorClockUtils};
pub use lww_element_set::{LWWElement, LWWElementSet};
pub use or_set::{ORSet, ORSetElement, ORSetTag};
