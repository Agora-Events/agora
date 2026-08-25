/**
 * Observed-Remove Set (OR-Set) CRDT
 * A set that handles concurrent adds and removes correctly
 * Each element is tagged with unique tags that are tracked separately
 */

import { VectorClock, VectorClockUtils } from './vectorClock';

export interface ORSetTag {
  id: string;
  nodeId: string;
  vectorClock: VectorClock;
}

export interface ORSetElement<T> {
  value: T;
  tags: ORSetTag[];
}

export class ORSet<T> {
  private elements: Map<string, ORSetElement<T>>;

  constructor() {
    this.elements = new Map();
  }

  /**
   * Generate a unique tag for an element
   */
  private generateTag(nodeId: string): ORSetTag {
    return {
      id: `${nodeId}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      nodeId,
      vectorClock: VectorClockUtils.create(nodeId, 1),
    };
  }

  /**
   * Generate a unique key for an element value
   */
  private generateKey(value: T): string {
    return JSON.stringify(value);
  }

  /**
   * Add an element to the set
   */
  add(value: T, nodeId: string): void {
    const key = this.generateKey(value);
    const tag = this.generateTag(nodeId);
    
    const existing = this.elements.get(key);
    
    if (existing) {
      // Add new tag to existing element
      existing.tags.push(tag);
    } else {
      // Create new element with tag
      this.elements.set(key, {
        value,
        tags: [tag],
      });
    }
  }

  /**
   * Remove an element from the set
   */
  remove(value: T): void {
    const key = this.generateKey(value);
    const element = this.elements.get(key);
    
    if (element) {
      // Remove all tags to delete the element
      element.tags = [];
      this.elements.delete(key);
    }
  }

  /**
   * Check if an element is in the set
   */
  has(value: T): boolean {
    const key = this.generateKey(value);
    const element = this.elements.get(key);
    
    if (!element) return false;
    return element.tags.length > 0;
  }

  /**
   * Get all elements in the set
   */
  values(): T[] {
    const result: T[] = [];
    
    for (const element of this.elements.values()) {
      if (element.tags.length > 0) {
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
      if (element.tags.length > 0) {
        count++;
      }
    }
    
    return count;
  }

  /**
   * Merge another OR-Set into this one
   */
  merge(other: ORSet<T>): void {
    for (const [key, otherElement] of other.elements.entries()) {
      const existing = this.elements.get(key);
      
      if (!existing) {
        // Element doesn't exist locally, copy it
        this.elements.set(key, {
          value: otherElement.value,
          tags: [...otherElement.tags],
        });
      } else {
        // Merge tags from both sets
        const mergedTags = this.mergeTags(existing.tags, otherElement.tags);
        
        if (mergedTags.length > 0) {
          this.elements.set(key, {
            value: existing.value,
            tags: mergedTags,
          });
        } else {
          // All tags were removed, delete the element
          this.elements.delete(key);
        }
      }
    }
  }

  /**
   * Merge tags from two sets, removing duplicates
   */
  private mergeTags(tags1: ORSetTag[], tags2: ORSetTag[]): ORSetTag[] {
    const tagMap = new Map<string, ORSetTag>();
    
    // Add all tags from first set
    for (const tag of tags1) {
      tagMap.set(tag.id, tag);
    }
    
    // Add or update tags from second set
    for (const tag of tags2) {
      const existing = tagMap.get(tag.id);
      
      if (existing) {
        // Merge vector clocks
        const mergedClock = VectorClockUtils.merge(
          existing.vectorClock,
          tag.vectorClock
        );
        tagMap.set(tag.id, {
          ...tag,
          vectorClock: mergedClock,
        });
      } else {
        tagMap.set(tag.id, tag);
      }
    }
    
    return Array.from(tagMap.values());
  }

  /**
   * Get the current state as a plain object
   */
  toJSON(): { elements: ORSetElement<T>[] } {
    return {
      elements: Array.from(this.elements.values()),
    };
  }

  /**
   * Create an OR-Set from a plain object
   */
  static fromJSON<T>(json: { elements: ORSetElement<T>[] }): ORSet<T> {
    const set = new ORSet<T>();
    
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
