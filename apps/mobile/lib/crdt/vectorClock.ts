/**
 * Vector Clock implementation for CRDT operations
 * Tracks causality between distributed operations
 */

export interface VectorClock {
  [nodeId: string]: number;
}

export class VectorClockUtils {
  /**
   * Create a new vector clock with initial value for a node
   */
  static create(nodeId: string, initialValue: number = 0): VectorClock {
    return {
      [nodeId]: initialValue,
    };
  }

  /**
   * Increment the counter for a specific node
   */
  static increment(clock: VectorClock, nodeId: string): VectorClock {
    const newClock = { ...clock };
    newClock[nodeId] = (newClock[nodeId] || 0) + 1;
    return newClock;
  }

  /**
   * Merge two vector clocks (takes maximum for each node)
   */
  static merge(clock1: VectorClock, clock2: VectorClock): VectorClock {
    const merged: VectorClock = { ...clock1 };
    
    for (const [nodeId, counter] of Object.entries(clock2)) {
      merged[nodeId] = Math.max(merged[nodeId] || 0, counter);
    }
    
    return merged;
  }

  /**
   * Compare two vector clocks
   * Returns: -1 if clock1 < clock2, 1 if clock1 > clock2, 0 if concurrent
   */
  static compare(clock1: VectorClock, clock2: VectorClock): number {
    let lessThan = false;
    let greaterThan = false;
    
    const allNodeIds = new Set([
      ...Object.keys(clock1),
      ...Object.keys(clock2),
    ]);
    
    for (const nodeId of allNodeIds) {
      const c1 = clock1[nodeId] || 0;
      const c2 = clock2[nodeId] || 0;
      
      if (c1 < c2) {
        lessThan = true;
      } else if (c1 > c2) {
        greaterThan = true;
      }
    }
    
    if (lessThan && !greaterThan) return -1;
    if (greaterThan && !lessThan) return 1;
    return 0; // Concurrent or equal
  }

  /**
   * Check if clock1 happened before clock2
   */
  static happenedBefore(clock1: VectorClock, clock2: VectorClock): boolean {
    return this.compare(clock1, clock2) === -1;
  }

  /**
   * Check if two clocks are concurrent
   */
  static isConcurrent(clock1: VectorClock, clock2: VectorClock): boolean {
    return this.compare(clock1, clock2) === 0;
  }

  /**
   * Check if two clocks are equal
   */
  static equals(clock1: VectorClock, clock2: VectorClock): boolean {
    const allNodeIds = new Set([
      ...Object.keys(clock1),
      ...Object.keys(clock2),
    ]);
    
    for (const nodeId of allNodeIds) {
      if ((clock1[nodeId] || 0) !== (clock2[nodeId] || 0)) {
        return false;
      }
    }
    
    return true;
  }

  /**
   * Serialize vector clock to JSON string
   */
  static serialize(clock: VectorClock): string {
    return JSON.stringify(clock);
  }

  /**
   * Deserialize vector clock from JSON string
   */
  static deserialize(json: string): VectorClock {
    return JSON.parse(json);
  }

  /**
   * Get all node IDs from a vector clock
   */
  static getNodeIds(clock: VectorClock): string[] {
    return Object.keys(clock);
  }

  /**
   * Get the counter value for a specific node
   */
  static getCounter(clock: VectorClock, nodeId: string): number {
    return clock[nodeId] || 0;
  }
}
