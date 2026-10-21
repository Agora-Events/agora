"use client";

// ─── Types ────────────────────────────────────────────────────────────────────

interface CacheEntry<T> {
  value: T;
  expiresAt: number;
}

interface CacheStats {
  hits: number;
  misses: number;
  size: number;
}

// ─── LRU / TTL cache implementation ──────────────────────────────────────────

/**
 * A simple in-memory LRU cache with per-entry TTL support.
 *
 * Used to avoid redundant Soroban RPC calls for contract metadata (tier
 * inventory, pricing) that rarely changes within a user session. Completing
 * a ticket purchase should immediately invalidate the relevant entries via
 * `invalidate()` or `invalidateAll()`.
 *
 * Default TTL: 30 seconds — enough to cache repeated modal opens while still
 * surfacing real-time inventory changes within a reasonable window.
 *
 * @template T - The type of values stored in the cache.
 *
 * @example
 * ```ts
 * const cache = new ContractCache<TierData>(30_000, 50);
 *
 * // On contract query:
 * const cached = cache.get(cacheKey);
 * if (cached) return cached;
 * const fresh = await fetchFromRpc(cacheKey);
 * cache.set(cacheKey, fresh);
 * return fresh;
 *
 * // After successful purchase:
 * cache.invalidate(cacheKey);
 * ```
 */
export class ContractCache<T = unknown> {
  /** Maximum number of entries before evicting the least-recently-used entry. */
  private readonly maxSize: number;
  /** Default time-to-live in milliseconds. */
  private readonly defaultTtl: number;

  /**
   * Insertion-ordered map: iterating yields entries from oldest to newest.
   * We exploit this for O(1) LRU eviction: the first entry is always the LRU.
   */
  private readonly store = new Map<string, CacheEntry<T>>();

  private stats: CacheStats = { hits: 0, misses: 0, size: 0 };

  constructor(defaultTtlMs = 30_000, maxSize = 100) {
    this.defaultTtl = defaultTtlMs;
    this.maxSize = maxSize;
  }

  // ── Read ──────────────────────────────────────────────────────────────────

  /**
   * Returns the cached value for `key`, or `undefined` if absent or expired.
   * Moves the entry to the "most recently used" position on a hit.
   */
  get(key: string): T | undefined {
    const entry = this.store.get(key);
    if (!entry) {
      this.stats.misses++;
      return undefined;
    }

    if (Date.now() > entry.expiresAt) {
      // Expired — evict lazily
      this.store.delete(key);
      this.stats.size = this.store.size;
      this.stats.misses++;
      return undefined;
    }

    // Refresh LRU position
    this.store.delete(key);
    this.store.set(key, entry);
    this.stats.hits++;
    return entry.value;
  }

  /**
   * Returns `true` if the key exists and has not expired.
   */
  has(key: string): boolean {
    return this.get(key) !== undefined;
  }

  // ── Write ─────────────────────────────────────────────────────────────────

  /**
   * Stores `value` under `key` with an optional per-entry TTL override.
   * If the cache is at capacity, the least-recently-used entry is evicted.
   */
  set(key: string, value: T, ttlMs?: number): void {
    // If the key already exists, remove it first so it becomes the MRU entry.
    if (this.store.has(key)) {
      this.store.delete(key);
    } else if (this.store.size >= this.maxSize) {
      // Evict the oldest (first) entry
      const firstKey = this.store.keys().next().value;
      if (firstKey !== undefined) {
        this.store.delete(firstKey);
      }
    }

    this.store.set(key, {
      value,
      expiresAt: Date.now() + (ttlMs ?? this.defaultTtl),
    });
    this.stats.size = this.store.size;
  }

  // ── Invalidation ──────────────────────────────────────────────────────────

  /**
   * Removes a specific entry, forcing the next access to fetch fresh data.
   * Call this after a successful ticket purchase to ensure inventory reflects
   * the latest on-chain state.
   */
  invalidate(key: string): boolean {
    const deleted = this.store.delete(key);
    this.stats.size = this.store.size;
    return deleted;
  }

  /**
   * Removes all entries whose keys start with `prefix`.
   * Useful for invalidating all tiers of a specific event at once.
   *
   * @example
   * ```ts
   * cache.invalidateByPrefix(`event:${eventId}:`);
   * ```
   */
  invalidateByPrefix(prefix: string): number {
    let count = 0;
    for (const key of this.store.keys()) {
      if (key.startsWith(prefix)) {
        this.store.delete(key);
        count++;
      }
    }
    this.stats.size = this.store.size;
    return count;
  }

  /**
   * Clears all cached entries.
   */
  invalidateAll(): void {
    this.store.clear();
    this.stats.size = 0;
  }

  // ── Diagnostics ───────────────────────────────────────────────────────────

  /**
   * Returns a snapshot of cache hit/miss/size counters.
   * Useful for developer tooling and performance assertions in tests.
   */
  getStats(): Readonly<CacheStats> {
    return { ...this.stats };
  }

  /**
   * Current number of non-expired entries (approximation — does not purge
   * stale entries before counting).
   */
  get size(): number {
    return this.store.size;
  }
}

// ─── Shared singletons ────────────────────────────────────────────────────────

/**
 * Shared cache for Soroban contract tier metadata (pricing + inventory).
 *
 * - TTL: 30 seconds — balances freshness against RPC rate-limit headroom.
 * - Max size: 200 entries (each entry ≈ one (eventId, tierId) pair).
 *
 * After a successful purchase call `tierMetadataCache.invalidateByPrefix()`
 * with the relevant event prefix to force fresh data on the next modal open.
 */
export const tierMetadataCache = new ContractCache<unknown>(30_000, 200);

// ─── Cache key helpers ────────────────────────────────────────────────────────

/**
 * Builds a deterministic cache key for a specific event's tier data.
 *
 * @example
 * ```ts
 * const key = tierCacheKey("evt_123", "tier_vip");
 * // → "tier:evt_123:tier_vip"
 * ```
 */
export function tierCacheKey(eventId: string, tierId: string): string {
  return `tier:${eventId}:${tierId}`;
}

/**
 * Builds a cache key for an event's full tier list.
 *
 * @example
 * ```ts
 * const key = eventTiersCacheKey("evt_123");
 * // → "tiers:evt_123"
 * ```
 */
export function eventTiersCacheKey(eventId: string): string {
  return `tiers:${eventId}`;
}

/**
 * Invalidates all tier-related cache entries for an event.
 * Call this immediately after a successful ticket purchase.
 *
 * @example
 * ```ts
 * invalidateEventCache("evt_123");
 * ```
 */
export function invalidateEventCache(eventId: string): void {
  tierMetadataCache.invalidateByPrefix(`tier:${eventId}:`);
  tierMetadataCache.invalidate(eventTiersCacheKey(eventId));
}
