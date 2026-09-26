/**
 * Observed-Remove Set (OR-Set) CRDT
 * A set that handles concurrent adds and removes correctly
 */
use crate::crdt::vector_clock::VectorClock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A tag for an OR-Set element
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ORSetTag {
    pub id: String,
    pub node_id: String,
    pub vector_clock: VectorClock,
}

/// An element in an OR-Set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSetElement<T> {
    pub value: T,
    pub tags: Vec<ORSetTag>,
}

/// Observed-Remove Set CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSet<T> {
    elements: HashMap<String, ORSetElement<T>>,
}

impl<T: Clone + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>> ORSet<T> {
    /// Create a new OR-Set
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    /// Generate a unique tag for an element
    fn generate_tag(node_id: &str) -> ORSetTag {
        let id = format!(
            "{}-{}-{}",
            node_id,
            chrono::Utc::now().timestamp_millis(),
            uuid::Uuid::new_v4()
        );
        ORSetTag {
            id,
            node_id: node_id.to_string(),
            vector_clock: VectorClock::new(node_id, 1),
        }
    }

    /// Generate a unique key for an element value
    fn generate_key(value: &T) -> String {
        serde_json::to_string(value).unwrap_or_default()
    }

    /// Add an element to the set
    pub fn add(&mut self, value: T, node_id: &str) {
        let key = Self::generate_key(&value);
        let tag = Self::generate_tag(node_id);

        if let Some(element) = self.elements.get_mut(&key) {
            element.tags.push(tag);
        } else {
            self.elements.insert(
                key,
                ORSetElement {
                    value,
                    tags: vec![tag],
                },
            );
        }
    }

    /// Remove an element from the set
    pub fn remove(&mut self, value: &T) {
        let key = Self::generate_key(value);
        self.elements.remove(&key);
    }

    /// Check if an element is in the set
    pub fn has(&self, value: &T) -> bool {
        let key = Self::generate_key(value);
        if let Some(element) = self.elements.get(&key) {
            !element.tags.is_empty()
        } else {
            false
        }
    }

    /// Get all elements in the set
    pub fn values(&self) -> Vec<T> {
        let mut result = Vec::new();
        for element in self.elements.values() {
            if !element.tags.is_empty() {
                result.push(element.value.clone());
            }
        }
        result
    }

    /// Get the size of the set
    pub fn size(&self) -> usize {
        self.elements
            .values()
            .filter(|e| !e.tags.is_empty())
            .count()
    }

    /// Merge another OR-Set into this one
    pub fn merge(&mut self, other: &ORSet<T>) {
        for (key, other_element) in &other.elements {
            if let Some(existing) = self.elements.get_mut(key) {
                let merged_tags = Self::merge_tags(&existing.tags, &other_element.tags);
                if merged_tags.is_empty() {
                    self.elements.remove(key);
                } else {
                    existing.tags = merged_tags;
                }
            } else {
                self.elements.insert(key.clone(), other_element.clone());
            }
        }
    }

    /// Merge tags from two sets, removing duplicates
    fn merge_tags(tags1: &[ORSetTag], tags2: &[ORSetTag]) -> Vec<ORSetTag> {
        let mut tag_map: HashMap<String, ORSetTag> = HashMap::new();

        // Add all tags from first set
        for tag in tags1 {
            tag_map.insert(tag.id.clone(), tag.clone());
        }

        // Add or update tags from second set
        for tag in tags2 {
            if let Some(existing) = tag_map.get_mut(&tag.id) {
                // Merge vector clocks
                existing.vector_clock = existing.vector_clock.merge(&tag.vector_clock);
            } else {
                tag_map.insert(tag.id.clone(), tag.clone());
            }
        }

        tag_map.into_values().collect()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl<T: Clone + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>> Default
    for ORSet<T>
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_or_set_add() {
        let mut set = ORSet::new();
        set.add("value1".to_string(), "node1");
        assert!(set.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_remove() {
        let mut set = ORSet::new();
        set.add("value1".to_string(), "node1");
        set.remove(&"value1".to_string());
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_merge() {
        let mut set1 = ORSet::new();
        set1.add("value1".to_string(), "node1");

        let mut set2 = ORSet::new();
        set2.add("value2".to_string(), "node2");

        set1.merge(&set2);
        assert!(set1.has(&"value1".to_string()));
        assert!(set1.has(&"value2".to_string()));
    }
}
