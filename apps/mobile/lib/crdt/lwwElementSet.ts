/**
 * Last-Write-Wins Element-Set (LWW-Element-Set) CRDT
 * A set where each element has a timestamp, and the latest write wins
 */

import { VectorClock, VectorClockUtils } from './vectorClock';

export interface LWWElement<T> {
  value: T;
  timestamp: number;
  vectorClock: VectorClock;
  isAdd: boolean; // true for add, false for remove
}

export class LWWElementSet<T> {
  private elements: Map<string, LWWElement<T>>;

  constructor() {
    this.elements = new Map();
  }

  /**
   * Generate a unique key for an element
   */
  private generateKey(value: T): string {
    return JSON.stringify(value);
  }

  /**
   * Add an element to the set
   */
  add(value: T, nodeId: string, timestamp?: number): void {
    const key = this.generateKey(value);
    const effectiveTimestamp = timestamp ?? Date.now();
    const vectorClock = VectorClockUtils.create(nodeId, 1);

    const existing = this.elements.get(key);
    
    // If element doesn't exist, or new operation wins based on timestamp
    if (!existing || effectiveTimestamp > existing.timestamp) {
      this.elements.set(key, {
        value,
        timestamp: effectiveTimestamp,
        vectorClock,
        isAdd: true,
      });
    } else if (effectiveTimestamp === existing.timestamp) {
      // If timestamps are equal, use vector clock to break tie
      const comparison = VectorClockUtils.compare(
        vectorClock,
        existing.vectorClock
      );
      
      if (comparison > 0) {
        this.elements.set(key, {
          value,
          timestamp: effectiveTimestamp,
          vectorClock,
          isAdd: true,
        });
      }
    }
  }

  /**
   * Remove an element from the set
   */
  remove(value: T, nodeId: string, timestamp?: number): void {
    const key = this.generateKey(value);
    const effectiveTimestamp = timestamp ?? Date.now();
    const vectorClock = VectorClockUtils.create(nodeId, 1);

    const existing = this.elements.get(key);
    
    // If element doesn't exist, or new operation wins based on timestamp
    if (!existing || effectiveTimestamp > existing.timestamp) {
      this.elements.set(key, {
        value,
        timestamp: effectiveTimestamp,
        vectorClock,
        isAdd: false,
      });
    } else if (effectiveTimestamp === existing.timestamp) {
      // If timestamps are equal, use vector clock to break tie
      const comparison = VectorClockUtils.compare(
        vectorClock,
        existing.vectorClock
      );
      
      if (comparison > 0) {
        this.elements.set(key, {
          value,
          timestamp: effectiveTimestamp,
          vectorClock,
          isAdd: false,
        });
      }
    }
  }

  /**
   * Check if an element is in the set
   */
  has(value: T): boolean {
    const key = this.generateKey(value);
    const element = this.elements.get(key);
    
    if (!element) return false;
    return element.isAdd;
  }

  /**
   * Get all elements in the set
   */
  values(): T[] {
    const result: T[] = [];
    
    for (const element of this.elements.values()) {
      if (element.isAdd) {
        result.push(element.value);
      }
    }
    
    return result;
  }

  /**
   * Get the size of the set
   */
  size(): number {
    let count = 0;
    
    for (const element of this.elements.values()) {
      if (element.isAdd) {
        count++;
      }
    }
    
    return count;
  }

  /**
   * Merge another LWW-Element-Set into this one
   */
  merge(other: LWWElementSet<T>): void {
    for (const [key, otherElement] of other.elements.entries()) {
      const existing = this.elements.get(key);
      
      if (!existing) {
        this.elements.set(key, otherElement);
      } else {
        // Compare timestamps
        if (otherElement.timestamp > existing.timestamp) {
          this.elements.set(key, otherElement);
        } else if (otherElement.timestamp === existing.timestamp) {
          // If timestamps are equal, merge vector clocks and compare
          const mergedClock = VectorClockUtils.merge(
            existing.vectorClock,
            otherElement.vectorClock
          );
          
          const comparison = VectorClockUtils.compare(
            otherElement.vectorClock,
            existing.vectorClock
          );
          
          if (comparison > 0) {
            this.elements.set(key, {
              ...otherElement,
              vectorClock: mergedClock,
            });
          } else if (comparison === 0) {
            // Concurrent with same timestamp - prefer add over remove
            if (otherElement.isAdd && !existing.isAdd) {
              this.elements.set(key, {
                ...otherElement,
                vectorClock: mergedClock,
              });
            }
          }
        }
      }
    }
  }

  /**
   * Get the current state as a plain object
   */
  toJSON(): { elements: LWWElement<T>[] } {
    return {
      elements: Array.from(this.elements.values()),
    };
  }

  /**
   * Create an LWW-Element-Set from a plain object
   */
  static fromJSON<T>(json: { elements: LWWElement<T>[] }): LWWElementSet<T> {
    const set = new LWWElementSet<T>();
    
    for (const element of json.elements) {
      const key = JSON.stringify(element.value);
      set.elements.set(key, element);
    }
    
    return set;
  }

  /**
   * Clear all elements
   */
  clear(): void {
    this.elements.clear();
  }
}
