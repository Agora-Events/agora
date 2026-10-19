pub mod lww_element_set;
pub mod or_set;
/**
 * CRDT module for server-side conflict resolution
 * Provides Vector Clock, LWW-Element-Set, and OR-Set implementations
 */
pub mod vector_clock;

#[cfg(test)]
mod tests;

pub use lww_element_set::{LWWElement, LWWElementSet};
pub use or_set::{ORSet, ORSetElement, ORSetTag};
pub use vector_clock::{VectorClock, VectorClockUtils};
