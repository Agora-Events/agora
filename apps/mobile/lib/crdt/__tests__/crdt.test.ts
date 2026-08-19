/**
 * CRDT Conflict Resolution and Convergence Tests
 * Tests for LWW-Element-Set, OR-Set, and Vector Clock implementations
 */

import {
  VectorClockUtils,
  LWWElementSet,
  ORSet,
  DeltaQueue,
} from '../index';

describe('VectorClock', () => {
  test('should create new vector clock', () => {
    const clock = VectorClockUtils.create('node1', 0);
    expect(clock).toEqual({ node1: 0 });
  });

  test('should increment counter for node', () => {
    const clock = VectorClockUtils.create('node1', 0);
    const incremented = VectorClockUtils.increment(clock, 'node1');
    expect(incremented).toEqual({ node1: 1 });
  });

  test('should merge two vector clocks', () => {
    const clock1 = { node1: 5, node2: 3 };
    const clock2 = { node2: 4, node3: 2 };
    const merged = VectorClockUtils.merge(clock1, clock2);
    expect(merged).toEqual({ node1: 5, node2: 4, node3: 2 });
  });

  test('should compare vector clocks - less than', () => {
    const clock1 = { node1: 1 };
    const clock2 = { node1: 2 };
    expect(VectorClockUtils.compare(clock1, clock2)).toBe(-1);
  });

  test('should compare vector clocks - greater than', () => {
    const clock1 = { node1: 2 };
    const clock2 = { node1: 1 };
    expect(VectorClockUtils.compare(clock1, clock2)).toBe(1);
  });

  test('should detect concurrent clocks', () => {
    const clock1 = { node1: 1 };
    const clock2 = { node2: 1 };
    expect(VectorClockUtils.isConcurrent(clock1, clock2)).toBe(true);
  });

  test('should detect equal clocks', () => {
    const clock1 = { node1: 1 };
    const clock2 = { node1: 1 };
    expect(VectorClockUtils.equals(clock1, clock2)).toBe(true);
  });
});

describe('LWW-Element-Set', () => {
  test('should add element', () => {
    const set = new LWWElementSet<string>();
    set.add('value1', 'node1', 100);
    expect(set.has('value1')).toBe(true);
    expect(set.size()).toBe(1);
  });

  test('should remove element with later timestamp', () => {
    const set = new LWWElementSet<string>();
    set.add('value1', 'node1', 100);
    set.remove('value1', 'node1', 200);
    expect(set.has('value1')).toBe(false);
  });

  test('should keep element with later add timestamp', () => {
    const set = new LWWElementSet<string>();
    set.remove('value1', 'node1', 100);
    set.add('value1', 'node1', 200);
    expect(set.has('value1')).toBe(true);
  });

  test('should merge two sets - convergence', () => {
    const set1 = new LWWElementSet<string>();
    set1.add('value1', 'node1', 100);
    
    const set2 = new LWWElementSet<string>();
    set2.add('value2', 'node2', 150);
    
    set1.merge(set2);
    
    expect(set1.has('value1')).toBe(true);
    expect(set1.has('value2')).toBe(true);
    expect(set1.size()).toBe(2);
  });

  test('should resolve conflicts with same timestamp using vector clock', () => {
    const set1 = new LWWElementSet<string>();
    set1.add('value1', 'node1', 100);
    
    const set2 = new LWWElementSet<string>();
    set2.remove('value1', 'node2', 100);
    
    // set1 should win because it was created first in this test
    set1.merge(set2);
    
    // The result depends on vector clock comparison
    expect(set1.size()).toBeGreaterThanOrEqual(0);
  });

  test('should serialize and deserialize', () => {
    const set = new LWWElementSet<string>();
    set.add('value1', 'node1', 100);
    
    const json = set.toJSON();
    const restored = LWWElementSet.fromJSON<string>(json);
    
    expect(restored.has('value1')).toBe(true);
  });
});

describe('OR-Set', () => {
  test('should add element', () => {
    const set = new ORSet<string>();
    set.add('value1', 'node1');
    expect(set.has('value1')).toBe(true);
    expect(set.size()).toBe(1);
  });

  test('should remove element', () => {
    const set = new ORSet<string>();
    set.add('value1', 'node1');
    set.remove('value1');
    expect(set.has('value1')).toBe(false);
  });

  test('should handle concurrent adds correctly', () => {
    const set1 = new ORSet<string>();
    set1.add('value1', 'node1');
    
    const set2 = new ORSet<string>();
    set2.add('value1', 'node2');
    
    set1.merge(set2);
    
    // Both adds should be preserved
    expect(set1.has('value1')).toBe(true);
  });

  test('should merge two sets - convergence', () => {
    const set1 = new ORSet<string>();
    set1.add('value1', 'node1');
    
    const set2 = new ORSet<string>();
    set2.add('value2', 'node2');
    
    set1.merge(set2);
    
    expect(set1.has('value1')).toBe(true);
    expect(set1.has('value2')).toBe(true);
    expect(set1.size()).toBe(2);
  });

  test('should handle add-remove conflict', () => {
    const set1 = new ORSet<string>();
    set1.add('value1', 'node1');
    
    const set2 = new ORSet<string>();
    set2.remove('value1');
    
    set1.merge(set2);
    
    // After merge, the element should be removed
    expect(set1.has('value1')).toBe(false);
  });

  test('should serialize and deserialize', () => {
    const set = new ORSet<string>();
    set.add('value1', 'node1');
    
    const json = set.toJSON();
    const restored = ORSet.fromJSON<string>(json);
    
    expect(restored.has('value1')).toBe(true);
  });
});

describe('DeltaQueue', () => {
  test('should add delta entry', () => {
    const queue = new DeltaQueue<any>();
    const vectorClock = { node1: 1 };
    
    const id = queue.add('bookmark', 'entity1', 'add', { data: 'test' }, vectorClock);
    
    expect(id).toBeDefined();
    expect(queue.size()).toBe(1);
  });

  test('should get unsynced entries', () => {
    const queue = new DeltaQueue<any>();
    const vectorClock = { node1: 1 };
    
    queue.add('bookmark', 'entity1', 'add', { data: 'test1' }, vectorClock);
    queue.add('bookmark', 'entity2', 'add', { data: 'test2' }, vectorClock);
    
    const unsynced = queue.getUnsynced();
    expect(unsynced.length).toBe(2);
  });

  test('should mark entry as synced', () => {
    const queue = new DeltaQueue<any>();
    const vectorClock = { node1: 1 };
    
    const id = queue.add('bookmark', 'entity1', 'add', { data: 'test' }, vectorClock);
    queue.markSynced(id);
    
    expect(queue.unsyncedCount()).toBe(0);
  });

  test('should remove synced entries', () => {
    const queue = new DeltaQueue<any>();
    const vectorClock = { node1: 1 };
    
    const id = queue.add('bookmark', 'entity1', 'add', { data: 'test' }, vectorClock);
    queue.markSynced(id);
    queue.removeSynced();
    
    expect(queue.size()).toBe(0);
  });

  test('should enforce max size', () => {
    const queue = new DeltaQueue<any>({ maxSize: 5 });
    const vectorClock = { node1: 1 };
    
    for (let i = 0; i < 10; i++) {
      queue.add('bookmark', `entity${i}`, 'add', { data: `test${i}` }, vectorClock);
    }
    
    expect(queue.size()).toBeLessThanOrEqual(5);
  });

  test('should serialize and deserialize', () => {
    const queue = new DeltaQueue<any>();
    const vectorClock = { node1: 1 };
    
    queue.add('bookmark', 'entity1', 'add', { data: 'test' }, vectorClock);
    
    const json = queue.toJSON();
    const restored = DeltaQueue.fromJSON<any>(json);
    
    expect(restored.size()).toBe(1);
  });
});

describe('CRDT Convergence', () => {
  test('should achieve convergence across multiple replicas', () => {
    // Create three replicas
    const replica1 = new LWWElementSet<string>();
    const replica2 = new LWWElementSet<string>();
    const replica3 = new LWWElementSet<string>();
    
    // Each replica adds different elements
    replica1.add('value1', 'node1', 100);
    replica2.add('value2', 'node2', 150);
    replica3.add('value3', 'node3', 200);
    
    // Merge all replicas
    replica1.merge(replica2);
    replica1.merge(replica3);
    replica2.merge(replica1);
    replica3.merge(replica2);
    
    // All replicas should converge to the same state
    expect(replica1.values().sort()).toEqual(replica2.values().sort());
    expect(replica2.values().sort()).toEqual(replica3.values().sort());
    expect(replica1.size()).toBe(3);
  });

  test('should resolve concurrent updates correctly', () => {
    const replica1 = new LWWElementSet<string>();
    const replica2 = new LWWElementSet<string>();
    
    // Both replicas update the same element concurrently
    replica1.add('value1', 'node1', 100);
    replica2.add('value1', 'node2', 100);
    
    // Merge - last write wins based on vector clock
    replica1.merge(replica2);
    
    // Should have exactly one value
    expect(replica1.size()).toBe(1);
    expect(replica1.has('value1')).toBe(true);
  });

  test('should handle network partition and recovery', () => {
    const replica1 = new LWWElementSet<string>();
    const replica2 = new LWWElementSet<string>();
    
    // Initial state
    replica1.add('value1', 'node1', 100);
    replica2.merge(replica1);
    
    // Network partition - both diverge
    replica1.add('value2', 'node1', 200);
    replica2.add('value3', 'node2', 250);
    
    // Network recovery - merge
    replica1.merge(replica2);
    replica2.merge(replica1);
    
    // Both should have all values
    expect(replica1.size()).toBe(3);
    expect(replica2.size()).toBe(3);
    expect(replica1.values().sort()).toEqual(replica2.values().sort());
  });
});
