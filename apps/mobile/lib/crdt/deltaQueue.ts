/**
 * Delta Queue for storing local mutations
 * Queues operations with unique vector clocks and transaction UUIDs
 * Supports IndexedDB/SQLite persistence
 */

import { VectorClock } from './vectorClock';

export type DeltaOperation = 'add' | 'remove' | 'update';

export interface DeltaEntry<T> {
  id: string; // Unique transaction UUID
  entityType: string; // e.g., 'event', 'attendee', 'bookmark'
  entityId: string;
  operation: DeltaOperation;
  value: T;
  vectorClock: VectorClock;
  timestamp: number;
  synced: boolean;
}

export interface DeltaQueueOptions {
  maxSize?: number;
  persistToStorage?: boolean;
  storageKey?: string;
}

export class DeltaQueue<T> {
  private queue: Map<string, DeltaEntry<T>>;
  private maxSize: number;
  private persistToStorage: boolean;
  private storageKey: string;

  constructor(options: DeltaQueueOptions = {}) {
    this.queue = new Map();
    this.maxSize = options.maxSize ?? 1000;
    this.persistToStorage = options.persistToStorage ?? false;
    this.storageKey = options.storageKey ?? 'delta_queue';

    if (this.persistToStorage) {
      this.loadFromStorage();
    }
  }

  /**
   * Generate a unique transaction UUID
   */
  private generateId(): string {
    return `tx-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Add a delta entry to the queue
   */
  add(
    entityType: string,
    entityId: string,
    operation: DeltaOperation,
    value: T,
    vectorClock: VectorClock
  ): string {
    const id = this.generateId();
    const entry: DeltaEntry<T> = {
      id,
      entityType,
      entityId,
      operation,
      value,
      vectorClock,
      timestamp: Date.now(),
      synced: false,
    };

    this.queue.set(id, entry);

    // Enforce max size by removing oldest entries
    if (this.queue.size > this.maxSize) {
      const oldestId = this.getOldestEntryId();
      if (oldestId) {
        this.queue.delete(oldestId);
      }
    }

    if (this.persistToStorage) {
      this.saveToStorage();
    }

    return id;
  }

  /**
   * Get a delta entry by ID
   */
  get(id: string): DeltaEntry<T> | undefined {
    return this.queue.get(id);
  }

  /**
   * Get all unsynced delta entries
   */
  getUnsynced(): DeltaEntry<T>[] {
    const unsynced: DeltaEntry<T>[] = [];
    
    for (const entry of this.queue.values()) {
      if (!entry.synced) {
        unsynced.push(entry);
      }
    }
    
    // Sort by timestamp for consistent ordering
    unsynced.sort((a, b) => a.timestamp - b.timestamp);
    
    return unsynced;
  }

  /**
   * Get all delta entries for a specific entity
   */
  getByEntity(entityType: string, entityId: string): DeltaEntry<T>[] {
    const entries: DeltaEntry<T>[] = [];
    
    for (const entry of this.queue.values()) {
      if (entry.entityType === entityType && entry.entityId === entityId) {
        entries.push(entry);
      }
    }
    
    return entries.sort((a, b) => a.timestamp - b.timestamp);
  }

  /**
   * Mark a delta entry as synced
   */
  markSynced(id: string): void {
    const entry = this.queue.get(id);
    if (entry) {
      entry.synced = true;
      
      if (this.persistToStorage) {
        this.saveToStorage();
      }
    }
  }

  /**
   * Mark multiple delta entries as synced
   */
  markSyncedBatch(ids: string[]): void {
    for (const id of ids) {
      const entry = this.queue.get(id);
      if (entry) {
        entry.synced = true;
      }
    }
    
    if (this.persistToStorage) {
      this.saveToStorage();
    }
  }

  /**
   * Remove synced entries from the queue
   */
  removeSynced(): void {
    const toDelete: string[] = [];
    
    for (const [id, entry] of this.queue.entries()) {
      if (entry.synced) {
        toDelete.push(id);
      }
    }
    
    for (const id of toDelete) {
      this.queue.delete(id);
    }
    
    if (this.persistToStorage) {
      this.saveToStorage();
    }
  }

  /**
   * Remove a specific entry from the queue
   */
  remove(id: string): void {
    this.queue.delete(id);
    
    if (this.persistToStorage) {
      this.saveToStorage();
    }
  }

  /**
   * Clear all entries from the queue
   */
  clear(): void {
    this.queue.clear();
    
    if (this.persistToStorage) {
      this.saveToStorage();
    }
  }

  /**
   * Get the size of the queue
   */
  size(): number {
    return this.queue.size;
  }

  /**
   * Get the number of unsynced entries
   */
  unsyncedCount(): number {
    let count = 0;
    
    for (const entry of this.queue.values()) {
      if (!entry.synced) {
        count++;
      }
    }
    
    return count;
  }

  /**
   * Get the oldest entry ID
   */
  private getOldestEntryId(): string | null {
    let oldestId: string | null = null;
    let oldestTimestamp = Infinity;
    
    for (const [id, entry] of this.queue.entries()) {
      if (entry.timestamp < oldestTimestamp) {
        oldestTimestamp = entry.timestamp;
        oldestId = id;
      }
    }
    
    return oldestId;
  }

  /**
   * Save queue to persistent storage (localStorage/AsyncStorage)
   */
  private saveToStorage(): void {
    try {
      const data = JSON.stringify(Array.from(this.queue.entries()));
      
      // For React Native, you'd use AsyncStorage here
      // For web, use localStorage
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(this.storageKey, data);
      }
    } catch (error) {
      console.error('Failed to save delta queue to storage:', error);
    }
  }

  /**
   * Load queue from persistent storage
   */
  private loadFromStorage(): void {
    try {
      let data: string | null = null;
      
      // For React Native, you'd use AsyncStorage here
      // For web, use localStorage
      if (typeof localStorage !== 'undefined') {
        data = localStorage.getItem(this.storageKey);
      }
      
      if (data) {
        const entries = JSON.parse(data) as [string, DeltaEntry<T>][];
        this.queue = new Map(entries);
      }
    } catch (error) {
      console.error('Failed to load delta queue from storage:', error);
    }
  }

  /**
   * Get the current state as a plain object
   */
  toJSON(): { entries: DeltaEntry<T>[] } {
    return {
      entries: Array.from(this.queue.values()),
    };
  }

  /**
   * Create a DeltaQueue from a plain object
   */
  static fromJSON<T>(json: { entries: DeltaEntry<T>[] }): DeltaQueue<T> {
    const queue = new DeltaQueue<T>();
    
    for (const entry of json.entries) {
      queue.queue.set(entry.id, entry);
    }
    
    return queue;
  }
}
