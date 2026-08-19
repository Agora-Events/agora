/**
 * CRDT Integration Tests for Conflict Resolution and Convergence
 * Tests for LWW-Element-Set, OR-Set, and Vector Clock implementations
 */

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::crdt::{LWWElementSet, ORSet, VectorClock, VectorClockUtils};

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
        let mut clock1 = VectorClock::new("node1", 5);
        clock1.counters.insert("node2".to_string(), 3);
        
        let clock2 = VectorClock::new("node2", 4);
        let merged = clock1.merge(&clock2);
        
        assert_eq!(merged.get_counter("node1"), 5);
        assert_eq!(merged.get_counter("node2"), 4);
    }

    #[test]
    fn test_vector_clock_compare_less_than() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node1", 2);
        assert_eq!(clock1.compare(&clock2), -1);
    }

    #[test]
    fn test_vector_clock_compare_greater_than() {
        let clock1 = VectorClock::new("node1", 2);
        let clock2 = VectorClock::new("node1", 1);
        assert_eq!(clock1.compare(&clock2), 1);
    }

    #[test]
    fn test_vector_clock_concurrent() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node2", 1);
        assert!(clock1.is_concurrent(&clock2));
    }

    #[test]
    fn test_vector_clock_equals() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node1", 1);
        assert!(clock1.equals(&clock2));
    }

    #[test]
    fn test_lww_element_set_add() {
        let mut set = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);
    }

    #[test]
    fn test_lww_element_set_remove() {
        let mut set = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        set.remove("value1".to_string(), "node1", Some(200));
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_lww_element_set_later_add_wins() {
        let mut set = LWWElementSet::new();
        set.remove("value1".to_string(), "node1", Some(100));
        set.add("value1".to_string(), "node1", Some(200));
        assert!(set.has(&"value1".to_string()));
    }

    #[test]
    fn test_lww_element_set_merge_convergence() {
        let mut set1 = LWWElementSet::new();
        set1.add("value1".to_string(), "node1", Some(100));
        
        let mut set2 = LWWElementSet::new();
        set2.add("value2".to_string(), "node2", Some(150));
        
        set1.merge(&set2);
        
        assert!(set1.has(&"value1".to_string()));
        assert!(set1.has(&"value2".to_string()));
        assert_eq!(set1.size(), 2);
    }

    #[test]
    fn test_lww_element_set_serialization() {
        let mut set = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        
        let json = serde_json::to_string(&set).unwrap();
        let restored: LWWElementSet<String> = serde_json::from_str(&json).unwrap();
        
        assert!(restored.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_add() {
        let mut set = ORSet::new();
        set.add("value1".to_string(), "node1");
        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);
    }

    #[test]
    fn test_or_set_remove() {
        let mut set = ORSet::new();
        set.add("value1".to_string(), "node1");
        set.remove(&"value1".to_string());
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_concurrent_adds() {
        let mut set1 = ORSet::new();
        set1.add("value1".to_string(), "node1");
        
        let mut set2 = ORSet::new();
        set2.add("value1".to_string(), "node2");
        
        set1.merge(&set2);
        
        // Both adds should be preserved
        assert!(set1.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_merge_convergence() {
        let mut set1 = ORSet::new();
        set1.add("value1".to_string(), "node1");
        
        let mut set2 = ORSet::new();
        set2.add("value2".to_string(), "node2");
        
        set1.merge(&set2);
        
        assert!(set1.has(&"value1".to_string()));
        assert!(set1.has(&"value2".to_string()));
        assert_eq!(set1.size(), 2);
    }

    #[test]
    fn test_or_set_add_remove_conflict() {
        let mut set1 = ORSet::new();
        set1.add("value1".to_string(), "node1");
        
        let mut set2 = ORSet::new();
        set2.remove(&"value1".to_string());
        
        set1.merge(&set2);
        
        // After merge, the element should be removed
        assert!(!set1.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_serialization() {
        let mut set = ORSet::new();
        set.add("value1".to_string(), "node1");
        
        let json = serde_json::to_string(&set).unwrap();
        let restored: ORSet<String> = serde_json::from_str(&json).unwrap();
        
        assert!(restored.has(&"value1".to_string()));
    }

    #[test]
    fn test_crdt_convergence_multiple_replicas() {
        // Create three replicas
        let mut replica1 = LWWElementSet::new();
        let mut replica2 = LWWElementSet::new();
        let mut replica3 = LWWElementSet::new();
        
        // Each replica adds different elements
        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.add("value2".to_string(), "node2", Some(150));
        replica3.add("value3".to_string(), "node3", Some(200));
        
        // Merge all replicas
        replica1.merge(&replica2);
        replica1.merge(&replica3);
        replica2.merge(&replica1);
        replica3.merge(&replica2);
        
        // All replicas should converge to the same state
        let mut values1 = replica1.values();
        let mut values2 = replica2.values();
        let mut values3 = replica3.values();
        
        values1.sort();
        values2.sort();
        values3.sort();
        
        assert_eq!(values1, values2);
        assert_eq!(values2, values3);
        assert_eq!(replica1.size(), 3);
    }

    #[test]
    fn test_crdt_concurrent_updates() {
        let mut replica1 = LWWElementSet::new();
        let mut replica2 = LWWElementSet::new();
        
        // Both replicas update the same element concurrently
        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.add("value1".to_string(), "node2", Some(100));
        
        // Merge - last write wins based on vector clock
        replica1.merge(&replica2);
        
        // Should have exactly one value
        assert_eq!(replica1.size(), 1);
        assert!(replica1.has(&"value1".to_string()));
    }

    #[test]
    fn test_crdt_network_partition_recovery() {
        let mut replica1 = LWWElementSet::new();
        let mut replica2 = LWWElementSet::new();
        
        // Initial state
        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.merge(&replica1);
        
        // Network partition - both diverge
        replica1.add("value2".to_string(), "node1", Some(200));
        replica2.add("value3".to_string(), "node2", Some(250));
        
        // Network recovery - merge
        replica1.merge(&replica2);
        replica2.merge(&replica1);
        
        // Both should have all values
        assert_eq!(replica1.size(), 3);
        assert_eq!(replica2.size(), 3);
        
        let mut values1 = replica1.values();
        let mut values2 = replica2.values();
        values1.sort();
        values2.sort();
        assert_eq!(values1, values2);
    }

    #[test]
    fn test_vector_clock_utils_merge() {
        let clock1 = VectorClock::new("node1", 5);
        let clock2 = VectorClock::new("node2", 3);
        let merged = VectorClockUtils::merge(&clock1, &clock2);
        
        assert_eq!(merged.get_counter("node1"), 5);
        assert_eq!(merged.get_counter("node2"), 3);
    }

    #[test]
    fn test_vector_clock_utils_compare() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node1", 2);
        assert_eq!(VectorClockUtils::compare(&clock1, &clock2), -1);
        assert_eq!(VectorClockUtils::compare(&clock2, &clock1), 1);
    }

    #[test]
    fn test_vector_clock_utils_happened_before() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node1", 2);
        assert!(VectorClockUtils::happened_before(&clock1, &clock2));
        assert!(!VectorClockUtils::happened_before(&clock2, &clock1));
    }

    #[test]
    fn test_vector_clock_utils_is_concurrent() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node2", 1);
        assert!(VectorClockUtils::is_concurrent(&clock1, &clock2));
    }

    #[test]
    fn test_vector_clock_utils_equals() {
        let clock1 = VectorClock::new("node1", 1);
        let clock2 = VectorClock::new("node1", 1);
        assert!(VectorClockUtils::equals(&clock1, &clock2));
    }
}
