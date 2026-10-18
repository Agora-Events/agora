/**
 * CRDT Integration Tests for Conflict Resolution and Convergence
 * Tests for LWW-Element-Set, OR-Set, and Vector Clock implementations
 */
#[cfg(test)]
mod integration_tests {
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
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);
    }

    #[test]
    fn test_lww_element_set_remove() {
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        set.remove("value1".to_string(), "node1", Some(200));
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_lww_element_set_later_add_wins() {
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        set.remove("value1".to_string(), "node1", Some(100));
        set.add("value1".to_string(), "node1", Some(200));
        assert!(set.has(&"value1".to_string()));
    }

    #[test]
    fn test_lww_element_set_merge_convergence() {
        let mut set1: LWWElementSet<String> = LWWElementSet::new();
        set1.add("value1".to_string(), "node1", Some(100));

        let mut set2: LWWElementSet<String> = LWWElementSet::new();
        set2.add("value2".to_string(), "node2", Some(150));

        set1.merge(&set2);

        assert!(set1.has(&"value1".to_string()));
        assert!(set1.has(&"value2".to_string()));
        assert_eq!(set1.size(), 2);
    }

    #[test]
    fn test_lww_element_set_serialization() {
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));

        let json = serde_json::to_string(&set).unwrap();
        let restored: LWWElementSet<String> = serde_json::from_str(&json).unwrap();

        assert!(restored.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_add() {
        let mut set: ORSet<String> = ORSet::new();
        set.add("value1".to_string(), "node1");
        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);
    }

    #[test]
    fn test_or_set_remove() {
        let mut set: ORSet<String> = ORSet::new();
        set.add("value1".to_string(), "node1");
        set.remove(&"value1".to_string());
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_concurrent_adds() {
        let mut set1: ORSet<String> = ORSet::new();
        set1.add("value1".to_string(), "node1");

        let mut set2: ORSet<String> = ORSet::new();
        set2.add("value1".to_string(), "node2");

        set1.merge(&set2);

        // Both adds should be preserved
        assert!(set1.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_merge_convergence() {
        let mut set1: ORSet<String> = ORSet::new();
        set1.add("value1".to_string(), "node1");

        let mut set2: ORSet<String> = ORSet::new();
        set2.add("value2".to_string(), "node2");

        set1.merge(&set2);

        assert!(set1.has(&"value1".to_string()));
        assert!(set1.has(&"value2".to_string()));
        assert_eq!(set1.size(), 2);
    }

    #[test]
    fn test_or_set_unobserved_concurrent_add_survives_remove() {
        let mut set1: ORSet<String> = ORSet::new();
        set1.add("value1".to_string(), "node1");

        let mut set2: ORSet<String> = ORSet::new();
        set2.add("value1".to_string(), "node2");
        set2.remove(&"value1".to_string());

        set1.merge(&set2);

        // node1's tag was never observed by node2, so node1's add survives:
        // an OR-Set may only drop the tags its remover actually saw.
        assert!(set1.has(&"value1".to_string()));

        // A remove-and-re-add on the same node starts a fresh tag again.
        set2.add("value1".to_string(), "node2");
        set1.merge(&set2);
        assert!(set1.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_serialization() {
        let mut set: ORSet<String> = ORSet::new();
        set.add("value1".to_string(), "node1");

        let json = serde_json::to_string(&set).unwrap();
        let restored: ORSet<String> = serde_json::from_str(&json).unwrap();

        assert!(restored.has(&"value1".to_string()));
    }

    #[test]
    fn test_crdt_convergence_multiple_replicas() {
        // Create three replicas
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();
        let mut replica3: LWWElementSet<String> = LWWElementSet::new();

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
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

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
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

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


    // Concurrency coverage for the sync path (issue #1668): concurrent updates,
    // timestamp monotonicity, and tombstone removal.

    #[test]
    fn test_crdt_concurrent_updates_later_write_wins() {
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

        // Same element, different nodes, replica2 writes later.
        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.add("value1".to_string(), "node2", Some(200));
        replica2.merge(&replica1);

        // replica1's write is staler and must not overwrite replica2's after merge.
        replica1.merge(&replica2);
        assert_eq!(replica1.size(), 1);
        assert!(replica2.has(&"value1".to_string()));
    }

    #[test]
    fn test_crdt_stale_add_cannot_resurrect_tombstone() {
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        set.remove("value1".to_string(), "node1", Some(300));

        // A re-add bearing a timestamp OLDER than the tombstone must not
        // resurrect the element — timestamps drive wins here, event order doesn't.
        set.add("value1".to_string(), "node1", Some(100));
        assert!(!set.has(&"value1".to_string()));

        // The right order: newer than the tombstone does resurrect.
        set.add("value1".to_string(), "node1", Some(400));
        assert!(set.has(&"value1".to_string()));
    }

    #[test]
    fn test_crdt_tombstone_survives_merge() {
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        replica1.add("value1".to_string(), "node1", Some(100));

        let mut replica2: LWWElementSet<String> = LWWElementSet::new();
        // replica2 only got the add; replica1 removed locally.
        replica2.merge(&replica1);
        replica1.remove("value1".to_string(), "node1", Some(200));

        // Merging the tombstone still hides the element on the other replica.
        replica2.merge(&replica1);
        assert!(!replica2.has(&"value1".to_string()));
        assert_eq!(replica2.size(), 0);

        // Re-add newer than the tombstone brings it back on the second replica.
        replica1.add("value1".to_string(), "node1", Some(300));
        replica2.merge(&replica1);
        assert!(replica2.has(&"value1".to_string()));
    }

    #[test]
    fn test_crdt_tombstone_from_merge_hides_value_on_both_sides() {
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.merge(&replica1);

        // replica2 removes; replica1 syncs and must not resurrect the value.
        replica2.remove("value1".to_string(), "node2", Some(200));
        replica1.merge(&replica2);
        assert!(!replica1.has(&"value1".to_string()));
        assert!(!replica2.has(&"value1".to_string()));
    }

    #[test]
    fn test_vector_clock_counters_are_monotonic_across_merges() {
        let mut clock1 = VectorClock::new("node1", 1);
        let mut clock2 = VectorClock::new("node2", 5);

        let before = clock1.get_counter("node1");
        let merged = clock1.merge(&clock2);
        let merged_back = clock2.merge(&merged);

        // Merging never decreases a counter on either side, in any order.
        assert_eq!(merged.get_counter("node1"), before);
        assert_eq!(merged.get_counter("node2"), 5);
        assert_eq!(merged_back.get_counter("node1"), before);
    }

    #[test]
    fn test_vector_clock_compare_with_empty_clock_is_lesser() {
        let clock1 = VectorClock::empty();
        let clock2 = VectorClock::new("node1", 3);
        assert_eq!(clock1.compare(&clock2), -1);
        assert_eq!(clock2.compare(&clock1), 1);
        assert!(clock1.happened_before(&clock2));
    }
}
