/**
 * Sync Engine for offline-first CRDT synchronization
 * Manages local mutations, network detection, and delta sync sessions
 */

import NetInfo, { NetInfoState } from '@react-native-community/netinfo';
import {
  VectorClock,
  VectorClockUtils,
  LWWElementSet,
  ORSet,
  DeltaQueue,
  DeltaEntry,
  DeltaOperation,
} from './crdt';

export interface SyncEngineConfig {
  nodeId: string;
  syncEndpoint: string;
  autoSync?: boolean;
  syncInterval?: number;
  maxRetries?: number;
  retryDelay?: number;
}

export interface SyncRequest {
  nodeId: string;
  vectorClock: VectorClock;
  deltas: DeltaEntry<any>[];
}

export interface SyncResponse {
  serverVectorClock: VectorClock;
  serverDeltas: DeltaEntry<any>[];
  mergedVectorClock: VectorClock;
}

export interface SyncState {
  isConnected: boolean;
  isSyncing: boolean;
  lastSyncTime: number | null;
  pendingDeltas: number;
  syncErrors: number;
}

export class SyncEngine {
  private config: SyncEngineConfig;
  private deltaQueue: DeltaQueue<any>;
  private localVectorClock: VectorClock;
  private syncState: SyncState;
  private syncTimer: ReturnType<typeof setInterval> | null = null;
  private unsubscribeNetInfo: (() => void) | null = null;

  // CRDT stores for different entity types
  private crdtStores: Map<string, LWWElementSet<any> | ORSet<any>>;

  constructor(config: SyncEngineConfig) {
    this.config = {
      autoSync: true,
      syncInterval: 30000, // 30 seconds
      maxRetries: 3,
      retryDelay: 5000, // 5 seconds
      ...config,
    };

    this.deltaQueue = new DeltaQueue<any>({
      maxSize: 1000,
      persistToStorage: true,
      storageKey: `delta_queue_${this.config.nodeId}`,
    });

    this.localVectorClock = VectorClockUtils.create(this.config.nodeId, 0);
    this.crdtStores = new Map();

    this.syncState = {
      isConnected: false,
      isSyncing: false,
      lastSyncTime: null,
      pendingDeltas: 0,
      syncErrors: 0,
    };

    this.initializeNetworkMonitoring();
  }

  /**
   * Initialize network state monitoring
   */
  private initializeNetworkMonitoring(): void {
    this.unsubscribeNetInfo = NetInfo.addEventListener((state: NetInfoState) => {
      const wasConnected = this.syncState.isConnected;
      this.syncState.isConnected = state.isConnected ?? false;

      // Trigger sync when coming back online
      if (!wasConnected && this.syncState.isConnected && this.config.autoSync) {
        this.triggerSync();
      }
    });

    // Check initial network state
    NetInfo.fetch().then((state: NetInfoState) => {
      this.syncState.isConnected = state.isConnected ?? false;
      if (this.syncState.isConnected && this.config.autoSync) {
        this.triggerSync();
      }
    });
  }

  /**
   * Get or create a CRDT store for an entity type
   */
  private getCRDTStore<T>(
    entityType: string,
    crdtType: 'lww' | 'or' = 'lww'
  ): LWWElementSet<T> | ORSet<T> {
    if (!this.crdtStores.has(entityType)) {
      const store =
        crdtType === 'lww' ? new LWWElementSet<T>() : new ORSet<T>();
      this.crdtStores.set(entityType, store);
    }
    return this.crdtStores.get(entityType)!;
  }

  /**
   * Queue a local mutation
   */
  queueMutation<T>(
    entityType: string,
    entityId: string,
    operation: DeltaOperation,
    value: T,
    crdtType: 'lww' | 'or' = 'lww'
  ): string {
    // Increment local vector clock
    this.localVectorClock = VectorClockUtils.increment(
      this.localVectorClock,
      this.config.nodeId
    );

    // Add to delta queue
    const deltaId = this.deltaQueue.add(
      entityType,
      entityId,
      operation,
      value,
      this.localVectorClock
    );

    // Apply to local CRDT store
    const store = this.getCRDTStore<T>(entityType, crdtType);
    
    if (crdtType === 'lww') {
      const lwwStore = store as LWWElementSet<T>;
      if (operation === 'add') {
        lwwStore.add(value, this.config.nodeId);
      } else if (operation === 'remove') {
        lwwStore.remove(value, this.config.nodeId);
      } else if (operation === 'update') {
        lwwStore.remove(value, this.config.nodeId);
        lwwStore.add(value, this.config.nodeId);
      }
    } else if (crdtType === 'or') {
      const orStore = store as ORSet<T>;
      if (operation === 'add') {
        orStore.add(value, this.config.nodeId);
      } else if (operation === 'remove') {
        orStore.remove(value);
      }
    }

    this.syncState.pendingDeltas = this.deltaQueue.unsyncedCount();

    // Trigger sync if connected
    if (this.syncState.isConnected && this.config.autoSync) {
      this.triggerSync();
    }

    return deltaId;
  }

  /**
   * Trigger a sync operation
   */
  private async triggerSync(): Promise<void> {
    if (this.syncState.isSyncing || !this.syncState.isConnected) {
      return;
    }

    this.syncState.isSyncing = true;

    try {
      await this.performSync();
    } catch (error) {
      console.error('Sync failed:', error);
      this.syncState.syncErrors++;
    } finally {
      this.syncState.isSyncing = false;
    }
  }

  /**
   * Perform the actual sync operation
   */
  private async performSync(): Promise<void> {
    const unsyncedDeltas = this.deltaQueue.getUnsynced();

    if (unsyncedDeltas.length === 0) {
      return;
    }

    const syncRequest: SyncRequest = {
      nodeId: this.config.nodeId,
      vectorClock: this.localVectorClock,
      deltas: unsyncedDeltas,
    };

    let retryCount = 0;
    let success = false;

    while (retryCount < this.config.maxRetries! && !success) {
      try {
        const response = await fetch(this.config.syncEndpoint, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(syncRequest),
        });

        if (!response.ok) {
          throw new Error(`Sync failed with status: ${response.status}`);
        }

        const syncResponse: SyncResponse = await response.json();
        await this.processSyncResponse(syncResponse);
        success = true;
      } catch (error) {
        retryCount++;
        if (retryCount < this.config.maxRetries!) {
          await this.delay(this.config.retryDelay!);
        } else {
          throw error;
        }
      }
    }
  }

  /**
   * Process sync response from server
   */
  private async processSyncResponse(response: SyncResponse): Promise<void> {
    // Merge server vector clock with local
    this.localVectorClock = VectorClockUtils.merge(
      this.localVectorClock,
      response.serverVectorClock
    );

    // Process server deltas
    for (const delta of response.serverDeltas) {
      const store = this.getCRDTStore(delta.entityType, 'lww'); // Default to LWW

      if (delta.operation === 'add') {
        if (store instanceof LWWElementSet) {
          const nodeId = Object.keys(delta.vectorClock)[0] || this.config.nodeId;
          store.add(delta.value, nodeId);
        } else if (store instanceof ORSet) {
          const nodeId = Object.keys(delta.vectorClock)[0] || this.config.nodeId;
          store.add(delta.value, nodeId);
        }
      } else if (delta.operation === 'remove') {
        if (store instanceof LWWElementSet) {
          const nodeId = Object.keys(delta.vectorClock)[0] || this.config.nodeId;
          store.remove(delta.value, nodeId);
        } else if (store instanceof ORSet) {
          store.remove(delta.value);
        }
      }
    }

    // Mark local deltas as synced
    const syncedIds = this.deltaQueue.getUnsynced().map((d) => d.id);
    this.deltaQueue.markSyncedBatch(syncedIds);
    this.deltaQueue.removeSynced();

    this.syncState.lastSyncTime = Date.now();
    this.syncState.pendingDeltas = this.deltaQueue.unsyncedCount();
  }

  /**
   * Start periodic sync
   */
  startPeriodicSync(): void {
    if (this.syncTimer) {
      clearInterval(this.syncTimer);
    }

    this.syncTimer = setInterval(() => {
      if (this.syncState.isConnected && this.config.autoSync) {
        this.triggerSync();
      }
    }, this.config.syncInterval!);
  }

  /**
   * Stop periodic sync
   */
  stopPeriodicSync(): void {
    if (this.syncTimer) {
      clearInterval(this.syncTimer);
      this.syncTimer = null;
    }
  }

  /**
   * Get current sync state
   */
  getSyncState(): SyncState {
    return { ...this.syncState };
  }

  /**
   * Get local vector clock
   */
  getVectorClock(): VectorClock {
    return { ...this.localVectorClock };
  }

  /**
   * Get CRDT store for an entity type
   */
  getStore<T>(entityType: string): LWWElementSet<T> | ORSet<T> | undefined {
    return this.crdtStores.get(entityType);
  }

  /**
   * Force sync regardless of network state
   */
  async forceSync(): Promise<void> {
    if (!this.syncState.isConnected) {
      throw new Error('Cannot sync while offline');
    }
    await this.triggerSync();
  }

  /**
   * Clear all data
   */
  clear(): void {
    this.deltaQueue.clear();
    this.crdtStores.clear();
    this.localVectorClock = VectorClockUtils.create(this.config.nodeId, 0);
    this.syncState = {
      isConnected: this.syncState.isConnected,
      isSyncing: false,
      lastSyncTime: null,
      pendingDeltas: 0,
      syncErrors: 0,
    };
  }

  /**
   * Cleanup and destroy the sync engine
   */
  destroy(): void {
    this.stopPeriodicSync();
    if (this.unsubscribeNetInfo) {
      this.unsubscribeNetInfo();
      this.unsubscribeNetInfo = null;
    }
  }

  /**
   * Delay helper for retry logic
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
