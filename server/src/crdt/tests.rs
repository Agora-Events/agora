/**
 * CRDT Integration Tests for Conflict Resolution and Convergence
 * Tests for LWW-Element-Set, OR-Set, and Vector Clock implementations
 */
#[cfg(test)]
mod integration_tests {
    use crate::crdt::{LWWElementSet, ORSet, VectorClock, VectorClockUtils};

    // ───────────────────────────────────────────────────────────────────────────
    // Vector Clock Tests (basic)
    // ───────────────────────────────────────────────────────────────────────────

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

    // ───────────────────────────────────────────────────────────────────────────
    // Timestamp Monotonicity Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_timestamp_monotonicity_increasing_wins() {
        // Later timestamp should always win regardless of node ID
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        
        // Add with timestamp 100
        set.add("value1".to_string(), "node1", Some(100));
        assert!(set.has(&"value1".to_string()));
        
        // Update with later timestamp 200
        set.add("value1".to_string(), "node1", Some(200));
        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);
        
        // Remove with even later timestamp 300
        set.remove("value1".to_string(), "node1", Some(300));
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_timestamp_monotonicity_non_monotonic_ignored() {
        // Earlier timestamp should be ignored
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        
        set.add("value1".to_string(), "node1", Some(200));
        
        // Try to update with earlier timestamp - should be ignored
        set.add("value1".to_string(), "node1", Some(100));
        
        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);
    }

    #[test]
    fn test_timestamp_monotonicity_concurrent_same_timestamp() {
        // When timestamps are equal, vector clock comparison breaks the tie
        let mut set1: LWWElementSet<String> = LWWElementSet::new();
        let mut set2: LWWElementSet<String> = LWWElementSet::new();

        // Add with same timestamp from different nodes
        set1.add("value1".to_string(), "node1", Some(100));
        set2.add("value1".to_string(), "node2", Some(100));

        // Merge - should resolve to one value based on vector clock comparison
        set1.merge(&set2);

        assert!(set1.has(&"value1".to_string()));
        assert_eq!(set1.size(), 1);
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Tombstone Removal Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_tombstone_add_then_remove() {
        // Add with timestamp 100, remove with timestamp 200
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        
        set.add("value1".to_string(), "node1", Some(100));
        assert!(set.has(&"value1".to_string()));
        
        set.remove("value1".to_string(), "node1", Some(200));
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_tombstone_remove_then_add() {
        // Remove with timestamp 100, add with timestamp 200
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        
        set.remove("value1".to_string(), "node1", Some(100));
        assert!(!set.has(&"value1".to_string()));
        
        set.add("value1".to_string(), "node1", Some(200));
        assert!(set.has(&"value1".to_string()));
    }

    #[test]
    fn test_tombstone_concurrent_add_remove_same_timestamp() {
        // When add and remove have same timestamp, add wins (conservative conflict resolution)
        let mut set1: LWWElementSet<String> = LWWElementSet::new();
        let mut set2: LWWElementSet<String> = LWWElementSet::new();

        // Node 1 adds
        set1.add("value1".to_string(), "node1", Some(100));
        // Node 2 removes (concurrent, same timestamp)
        set2.remove("value1".to_string(), "node2", Some(100));

        set1.merge(&set2);

        // Add should win due to tie-breaker preferring add over remove
        assert!(set1.has(&"value1".to_string()));
    }

    #[test]
    fn test_tombstone_remove_lower_timestamp() {
        // Remove with lower timestamp than add should be ignored
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        
        set.add("value1".to_string(), "node1", Some(200));
        set.remove("value1".to_string(), "node1", Some(100));
        
        assert!(set.has(&"value1".to_string()));
    }

    #[test]
    fn test_tombstone_remove_higher_timestamp() {
        // Remove with higher timestamp should work
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        
        set.add("value1".to_string(), "node1", Some(100));
        set.remove("value1".to_string(), "node1", Some(300));
        
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_tombstone_remove_by_different_node() {
        // Remove from different node but higher timestamp should work
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        
        set.add("value1".to_string(), "node1", Some(100));
        set.remove("value1".to_string(), "node2", Some(200));
        
        assert!(!set.has(&"value1".to_string()));
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Concurrent Update Tests
    // ───────────────────────────────────────────────────────────────────────────

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
    fn test_concurrent_updates_different_elements() {
        // Different elements added concurrently should both be present
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.add("value2".to_string(), "node2", Some(150));

        replica1.merge(&replica2);

        assert!(replica1.has(&"value1".to_string()));
        assert!(replica1.has(&"value2".to_string()));
        assert_eq!(replica1.size(), 2);
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
    fn test_concurrent_updates_same_element_different_nodes() {
        // Same element added concurrently from different nodes
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.add("value1".to_string(), "node2", Some(100));

        replica1.merge(&replica2);

        // Should have exactly one value (LWW resolution)
        assert_eq!(replica1.size(), 1);
        assert!(replica1.has(&"value1".to_string()));
    }

    #[test]
    fn test_concurrent_updates_merge_all_replicas() {
        // Three replicas with concurrent updates
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();
        let mut replica3: LWWElementSet<String> = LWWElementSet::new();

        // Different elements added to each replica
        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.add("value2".to_string(), "node2", Some(200));
        replica3.add("value3".to_string(), "node3", Some(300));

        // Merge all replicas together
        replica1.merge(&replica2);
        replica1.merge(&replica3);

        // All values should be present
        assert!(replica1.has(&"value1".to_string()));
        assert!(replica1.has(&"value2".to_string()));
        assert!(replica1.has(&"value3".to_string()));
        assert_eq!(replica1.size(), 3);
    }

    #[test]
    fn test_concurrent_updates_interleaved() {
        // Simulate interleaved updates between two replicas
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

        // Replica 1 adds value1
        replica1.add("value1".to_string(), "node1", Some(100));
        // Replica 2 adds value2
        replica2.add("value2".to_string(), "node2", Some(150));

        // Replica 1 merges
        replica1.merge(&replica2);
        assert_eq!(replica1.size(), 2);

        // Replica 2 merges later with different state
        replica2.merge(&replica1);
        assert_eq!(replica2.size(), 2);

        // Both should converge
        assert!(replica1.has(&"value1".to_string()));
        assert!(replica1.has(&"value2".to_string()));
        assert!(replica2.has(&"value1".to_string()));
        assert!(replica2.has(&"value2".to_string()));
    }

    #[test]
    fn test_concurrent_remove_operations() {
        // Concurrent removes from different nodes
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

        // Both add the same element
        replica1.add("value1".to_string(), "node1", Some(100));
        replica2.add("value1".to_string(), "node2", Some(100));

        // Both remove concurrently
        replica1.remove("value1".to_string(), "node1", Some(200));
        replica2.remove("value1".to_string(), "node2", Some(200));

        replica1.merge(&replica2);

        // Should be removed
        assert!(!replica1.has(&"value1".to_string()));
    }

    #[test]
    fn test_concurrent_add_remove_different_timestamps() {
        // Concurrent add and remove with different timestamps
        let mut replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut replica2: LWWElementSet<String> = LWWElementSet::new();

        // Replica 1 adds with timestamp 100
        replica1.add("value1".to_string(), "node1", Some(100));

        // Replica 2 adds and removes with higher timestamps
        replica2.add("value1".to_string(), "node2", Some(200));
        replica2.remove("value1".to_string(), "node2", Some(300));

        replica1.merge(&replica2);

        // Should be removed (remove with highest timestamp wins)
        assert!(!replica1.has(&"value1".to_string()));
    }

    // ───────────────────────────────────────────────────────────────────────────
    // OR-Set Concurrent Tests
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_or_set_concurrent_adds_different_nodes() {
        // Same element added concurrently from different nodes
        let mut set1: ORSet<String> = ORSet::new();
        let mut set2: ORSet<String> = ORSet::new();

        set1.add("value1".to_string(), "node1");
        set2.add("value1".to_string(), "node2");

        set1.merge(&set2);

        // OR-Set should preserve both adds
        assert!(set1.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_add_remove_by_same_node() {
        // Add then remove by the same node
        let mut set: ORSet<String> = ORSet::new();

        set.add("value1".to_string(), "node1");
        assert!(set.has(&"value1".to_string()));

        set.remove(&"value1".to_string());
        assert!(!set.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_remove_by_different_node() {
        // Add by node1, remove by node2
        let mut set1: ORSet<String> = ORSet::new();
        let mut set2: ORSet<String> = ORSet::new();

        set1.add("value1".to_string(), "node1");
        set2.remove(&"value1".to_string());

        set1.merge(&set2);

        // After merge, element should be removed
        assert!(!set1.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_multiple_concurrent_adds() {
        // Multiple concurrent adds should all be preserved
        let mut set1: ORSet<String> = ORSet::new();
        let mut set2: ORSet<String> = ORSet::new();
        let mut set3: ORSet<String> = ORSet::new();

        set1.add("value1".to_string(), "node1");
        set2.add("value1".to_string(), "node2");
        set3.add("value1".to_string(), "node3");

        set1.merge(&set2);
        set1.merge(&set3);

        assert!(set1.has(&"value1".to_string()));
    }

    #[test]
    fn test_or_set_cascading_merges() {
        // Test that merges propagate correctly through multiple replicas
        let mut replica1: ORSet<String> = ORSet::new();
        let mut replica2: ORSet<String> = ORSet::new();
        let mut replica3: ORSet<String> = ORSet::new();

        // Different adds on each replica
        replica1.add("value1".to_string(), "node1");
        replica2.add("value2".to_string(), "node2");
        replica3.add("value3".to_string(), "node3");

        // Cascading merges
        replica1.merge(&replica2);
        replica2.merge(&replica3);
        replica3.merge(&replica1);

        // All replicas should converge
        assert!(replica1.has(&"value1".to_string()));
        assert!(replica1.has(&"value2".to_string()));
        assert!(replica1.has(&"value3".to_string()));
        assert_eq!(replica1.size(), 3);

        assert_eq!(replica1.size(), replica2.size());
        assert_eq!(replica1.size(), replica3.size());
    }

    // ───────────────────────────────────────────────────────────────────────────
    // Vector Clock Utility Tests
    // ───────────────────────────────────────────────────────────────────────────

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

    // ───────────────────────────────────────────────────────────────────────────
    // Edge Cases and Complex Scenarios
    // ───────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_empty_merge() {
        // Merging with empty set should not change anything
        let mut set1: LWWElementSet<String> = LWWElementSet::new();
        let set2: LWWElementSet<String> = LWWElementSet::new();

        set1.add("value1".to_string(), "node1", Some(100));
        set1.merge(&set2);

        assert!(set1.has(&"value1".to_string()));
        assert_eq!(set1.size(), 1);
    }

    #[test]
    fn test_self_merge() {
        // Merging a set with itself should be idempotent
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        set.add("value2".to_string(), "node2", Some(200));

        let initial_size = set.size();
        set.merge(&set);

        assert_eq!(set.size(), initial_size);
    }

    #[test]
    fn test_lww_vs_or_set_convergence() {
        // Both CRDTs should converge to the same visible state
        let mut lww_set: LWWElementSet<String> = LWWElementSet::new();
        let mut or_set: ORSet<String> = ORSet::new();

        let mut lww_replica1: LWWElementSet<String> = LWWElementSet::new();
        let mut lww_replica2: LWWElementSet<String> = LWWElementSet::new();
        let mut or_replica1: ORSet<String> = ORSet::new();
        let mut or_replica2: ORSet<String> = ORSet::new();

        // Add same values to both types
        lww_replica1.add("value1".to_string(), "node1", Some(100));
        or_replica1.add("value1".to_string(), "node1");

        lww_replica2.add("value2".to_string(), "node2", Some(200));
        or_replica2.add("value2".to_string(), "node2");

        // Merge both types
        lww_replica1.merge(&lww_replica2);
        or_replica1.merge(&or_replica2);

        // Both should have 2 elements
        assert_eq!(lww_replica1.size(), 2);
        assert_eq!(or_replica1.size(), 2);
    }

    #[test]
    fn test_multiple_update_cycles() {
        // Multiple add/remove cycles with increasing timestamps
        let mut set: LWWElementSet<String> = LWWElementSet::new();

        // Cycle 1
        set.add("value1".to_string(), "node1", Some(100));
        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);

        // Cycle 2
        set.remove("value1".to_string(), "node1", Some(200));
        assert!(!set.has(&"value1".to_string()));
        assert_eq!(set.size(), 0);

        // Cycle 3 - add again
        set.add("value1".to_string(), "node1", Some(300));
        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);

        // Cycle 4 - remove with different node
        set.remove("value1".to_string(), "node2", Some(400));
        assert!(!set.has(&"value1".to_string()));
        assert_eq!(set.size(), 0);
    }

    #[test]
    fn test_node_id_independence() {
        // Same element from same node with higher timestamp should win
        let mut set: LWWElementSet<String> = LWWElementSet::new();

        set.add("value1".to_string(), "node1", Some(100));
        set.add("value1".to_string(), "node1", Some(200));

        assert!(set.has(&"value1".to_string()));
        assert_eq!(set.size(), 1);
    }

    #[test]
    fn test_tombstone_serialization_roundtrip() {
        // Test that tombstones are properly serialized/deserialized
        let mut set: LWWElementSet<String> = LWWElementSet::new();
        set.add("value1".to_string(), "node1", Some(100));
        set.remove("value1".to_string(), "node1", Some(200));

        let json = serde_json::to_string(&set).unwrap();
        let restored: LWWElementSet<String> = serde_json::from_str(&json).unwrap();

        assert!(!restored.has(&"value1".to_string()));
        assert_eq!(restored.size(), 0);
    }

    #[test]
    fn test_or_set_serialization_with_tags() {
        // Test OR-Set serialization preserves tags
        let mut set: ORSet<String> = ORSet::new();
        set.add("value1".to_string(), "node1");
        set.add("value2".to_string(), "node2");

        let json = serde_json::to_string(&set).unwrap();
        let restored: ORSet<String> = serde_json::from_str(&json).unwrap();

        assert!(restored.has(&"value1".to_string()));
        assert!(restored.has(&"value2".to_string()));
        assert_eq!(restored.size(), 2);
    }
}
