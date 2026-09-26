/**
 * CRDT module exports
 * Provides Conflict-Free Replicated Data Types for offline-first sync
 */

export { VectorClockUtils } from './vectorClock';
export type { VectorClock } from './vectorClock';

export { LWWElementSet } from './lwwElementSet';
export type { LWWElement } from './lwwElementSet';

export { ORSet } from './orSet';
export type { ORSetTag, ORSetElement } from './orSet';

export { DeltaQueue } from './deltaQueue';
export type { DeltaEntry, DeltaOperation, DeltaQueueOptions } from './deltaQueue';
