/**
 * Vector Clock implementation for CRDT operations
 * Tracks causality between distributed operations
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A vector clock is a map of node IDs to counters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    pub counters: HashMap<String, u64>,
}

impl VectorClock {
    /// Create a new vector clock with initial value for a node
    pub fn new(node_id: &str, initial_value: u64) -> Self {
        let mut counters = HashMap::new();
        counters.insert(node_id.to_string(), initial_value);
        Self { counters }
    }

    /// Create an empty vector clock
    pub fn empty() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    /// Increment the counter for a specific node
    pub fn increment(&mut self, node_id: &str) {
        let counter = self.counters.entry(node_id.to_string()).or_insert(0);
        *counter += 1;
    }

    /// Merge two vector clocks (takes maximum for each node)
    pub fn merge(&self, other: &VectorClock) -> VectorClock {
        let mut merged = self.counters.clone();
        
        for (node_id, counter) in &other.counters {
            let entry = merged.entry(node_id.clone()).or_insert(0);
            *entry = (*entry).max(*counter);
        }
        
        Self { counters: merged }
    }

    /// Compare two vector clocks
    /// Returns: -1 if self < other, 1 if self > other, 0 if concurrent or equal
    pub fn compare(&self, other: &VectorClock) -> i8 {
        let mut less_than = false;
        let mut greater_than = false;
        
        let all_node_ids: std::collections::HashSet<&String> = self.counters
            .keys()
            .chain(other.counters.keys())
            .collect();
        
        for node_id in all_node_ids {
            let c1 = self.counters.get(node_id).unwrap_or(&0);
            let c2 = other.counters.get(node_id).unwrap_or(&0);
            
            if c1 < c2 {
                less_than = true;
            } else if c1 > c2 {
                greater_than = true;
            }
        }
        
        if less_than && !greater_than {
            -1
        } else if greater_than && !less_than {
            1
        } else {
            0 // Concurrent or equal
        }
    }

    /// Check if self happened before other
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        self.compare(other) == -1
    }

    /// Check if two clocks are concurrent
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        self.compare(other) == 0 && !self.equals(other)
    }

    /// Check if two clocks are equal
    pub fn equals(&self, other: &VectorClock) -> bool {
        let all_node_ids: std::collections::HashSet<&String> = self.counters
            .keys()
            .chain(other.counters.keys())
            .collect();
        
        for node_id in all_node_ids {
            let c1 = self.counters.get(node_id).unwrap_or(&0);
            let c2 = other.counters.get(node_id).unwrap_or(&0);
            if c1 != c2 {
                return false;
            }
        }
        
        true
    }

    /// Get all node IDs from the vector clock
    pub fn node_ids(&self) -> Vec<String> {
        self.counters.keys().cloned().collect()
    }

    /// Get the counter value for a specific node
    pub fn get_counter(&self, node_id: &str) -> u64 {
        *self.counters.get(node_id).unwrap_or(&0)
    }
}

pub struct VectorClockUtils;

impl VectorClockUtils {
    /// Create a new vector clock with initial value for a node
    pub fn create(node_id: &str, initial_value: u64) -> VectorClock {
        VectorClock::new(node_id, initial_value)
    }

    /// Increment the counter for a specific node
    pub fn increment(clock: &VectorClock, node_id: &str) -> VectorClock {
        let mut new_clock = clock.clone();
        new_clock.increment(node_id);
        new_clock
    }

    /// Merge two vector clocks
    pub fn merge(clock1: &VectorClock, clock2: &VectorClock) -> VectorClock {
        clock1.merge(clock2)
    }

    /// Compare two vector clocks
    pub fn compare(clock1: &VectorClock, clock2: &VectorClock) -> i8 {
        clock1.compare(clock2)
    }

    /// Check if clock1 happened before clock2
    pub fn happened_before(clock1: &VectorClock, clock2: &VectorClock) -> bool {
        clock1.happened_before(clock2)
    }

    /// Check if two clocks are concurrent
    pub fn is_concurrent(clock1: &VectorClock, clock2: &VectorClock) -> bool {
        clock1.is_concurrent(clock2)
    }

    /// Check if two clocks are equal
    pub fn equals(clock1: &VectorClock, clock2: &VectorClock) -> bool {
        clock1.equals(clock2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_creation() {
        let clock = VectorClock::new("node1", 0);
        assert_eq!(clock.get_counter("node1"), 0);
    }

    #[test]
    fn test_vector_clock_increment() {
        let mut clock = VectorClock::new("node1", 0);
        clock.increment("node1");
        assert_eq!(clock.get_counter("node1"), 1);
    }

    #[test]
    fn test_vector_clock_merge() {
        let clock1 = VectorClock::new("node1", 5);
        let clock2 = VectorClock::new("node2", 3);
        let merged = clock1.merge(&clock2);
        assert_eq!(merged.get_counter("node1"), 5);
        assert_eq!(merged.get_counter("node2"), 3);
    }

    #[test]
    fn test_vector_clock_compare() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node1", 2);
        assert_eq!(clock1.compare(&clock2), -1);
        assert_eq!(clock2.compare(&clock1), 1);
    }

    #[test]
    fn test_vector_clock_concurrent() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node2", 1);
        assert!(clock1.is_concurrent(&clock2));
    }
}
