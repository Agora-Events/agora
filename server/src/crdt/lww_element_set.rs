/**
 * Last-Write-Wins Element-Set (LWW-Element-Set) CRDT
 * A set where each element has a timestamp, and the latest write wins
 */
use crate::crdt::vector_clock::VectorClock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An element in an LWW-Element-Set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWElement<T> {
    pub value: T,
    pub timestamp: i64,
    pub vector_clock: VectorClock,
    pub is_add: bool,
}

/// Last-Write-Wins Element-Set CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWElementSet<T> {
    elements: HashMap<String, LWWElement<T>>,
}

impl<T: Clone + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>> LWWElementSet<T> {
    /// Create a new LWW-Element-Set
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    /// Generate a unique key for an element
    fn generate_key(value: &T) -> String {
        serde_json::to_string(value).unwrap_or_default()
    }

    /// Add an element to the set
    pub fn add(&mut self, value: T, node_id: &str, timestamp: Option<i64>) {
        let key = Self::generate_key(&value);
        let effective_timestamp =
            timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
        let vector_clock = VectorClock::new(node_id, 1);

        if let Some(existing) = self.elements.get(&key) {
            if effective_timestamp > existing.timestamp {
                self.elements.insert(
                    key,
                    LWWElement {
                        value,
                        timestamp: effective_timestamp,
                        vector_clock,
                        is_add: true,
                    },
                );
            } else if effective_timestamp == existing.timestamp {
                let comparison = vector_clock.compare(&existing.vector_clock);
                if comparison > 0 {
                    self.elements.insert(
                        key,
                        LWWElement {
                            value,
                            timestamp: effective_timestamp,
                            vector_clock,
                            is_add: true,
                        },
                    );
                }
            }
        } else {
            self.elements.insert(
                key,
                LWWElement {
                    value,
                    timestamp: effective_timestamp,
                    vector_clock,
                    is_add: true,
                },
            );
        }
    }

    /// Remove an element from the set
    pub fn remove(&mut self, value: T, node_id: &str, timestamp: Option<i64>) {
        let key = Self::generate_key(&value);
        let effective_timestamp =
            timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
        let vector_clock = VectorClock::new(node_id, 1);

        if let Some(existing) = self.elements.get(&key) {
            if effective_timestamp > existing.timestamp {
                self.elements.insert(
                    key,
                    LWWElement {
                        value,
                        timestamp: effective_timestamp,
                        vector_clock,
                        is_add: false,
                    },
                );
            } else if effective_timestamp == existing.timestamp {
                let comparison = vector_clock.compare(&existing.vector_clock);
                if comparison > 0 {
                    self.elements.insert(
                        key,
                        LWWElement {
                            value,
                            timestamp: effective_timestamp,
                            vector_clock,
                            is_add: false,
                        },
                    );
                }
            }
        } else {
            self.elements.insert(
                key,
                LWWElement {
                    value,
                    timestamp: effective_timestamp,
                    vector_clock,
                    is_add: false,
                },
            );
        }
    }

    /// Check if an element is in the set
    pub fn has(&self, value: &T) -> bool {
        let key = Self::generate_key(value);
        if let Some(element) = self.elements.get(&key) {
            element.is_add
        } else {
            false
        }
    }

    /// Get all elements in the set
    pub fn values(&self) -> Vec<T> {
        let mut result = Vec::new();
        for element in self.elements.values() {
            if element.is_add {
                result.push(element.value.clone());
            }
        }
        result
    }

    /// Get the size of the set
    pub fn size(&self) -> usize {
        self.elements.values().filter(|e| e.is_add).count()
    }

    /// Merge another LWW-Element-Set into this one
    pub fn merge(&mut self, other: &LWWElementSet<T>) {
        for (key, other_element) in &other.elements {
            if let Some(existing) = self.elements.get(key) {
                if other_element.timestamp > existing.timestamp {
                    self.elements.insert(key.clone(), other_element.clone());
                } else if other_element.timestamp == existing.timestamp {
                    let merged_clock = existing.vector_clock.merge(&other_element.vector_clock);
                    let comparison = other_element.vector_clock.compare(&existing.vector_clock);

                    if comparison > 0 {
                        let mut element = other_element.clone();
                        element.vector_clock = merged_clock;
                        self.elements.insert(key.clone(), element);
                    } else if comparison == 0 {
                        // Concurrent with same timestamp - prefer add over remove
                        if other_element.is_add && !existing.is_add {
                            let mut element = other_element.clone();
                            element.vector_clock = merged_clock;
                            self.elements.insert(key.clone(), element);
                        }
                    }
                }
            } else {
                self.elements.insert(key.clone(), other_element.clone());
            }
        }
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl<T: Clone + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>> Default
    for LWWElementSet<T>
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_add() {
        let mut set = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        assert!(set.has(&"value1".to_string()));
    }

    #[test]
    fn test_lww_remove() {
        let mut set = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        set.remove("value1".to_string(), "node1", Some(200));
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_lww_merge() {
        let mut set1 = LWWElementSet::new();
        set1.add("value1".to_string(), "node1", Some(100));

        let mut set2 = LWWElementSet::new();
        set2.add("value2".to_string(), "node2", Some(150));

        set1.merge(&set2);
        assert!(set1.has(&"value1".to_string()));
        assert!(set1.has(&"value2".to_string()));
    }
}
