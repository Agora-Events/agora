/**
 * In-memory fixed-window rate limiter for Next.js route handlers (Issue #1586).
 *
 * Counters live in the server process, so limits apply per instance. That is
 * enough to blunt bot spam against the testnet deployment; a multi-instance
 * deployment should move the counters to Redis.
 */

export interface RateLimitResult {
  allowed: boolean;
  limit: number;
  remaining: number;
  /** Unix timestamp (ms) at which the current window ends. */
  resetAt: number;
  /** Seconds until the window ends, always >= 1. */
  retryAfterSeconds: number;
}

export interface RateLimiter {
  readonly limit: number;
  readonly windowMs: number;
  check(key: string, now?: number): RateLimitResult;
  reset(): void;
}

/** Expired entries are swept once the map grows past this many keys. */
const SWEEP_THRESHOLD = 10_000;

export function createRateLimiter({
  limit,
  windowMs,
}: {
  limit: number;
  windowMs: number;
}): RateLimiter {
  const windows = new Map<string, { count: number; resetAt: number }>();

  const sweep = (now: number) => {
    for (const [key, entry] of windows) {
      if (entry.resetAt <= now) windows.delete(key);
    }
  };

  return {
    limit,
    windowMs,
    check(key: string, now: number = Date.now()): RateLimitResult {
      if (windows.size > SWEEP_THRESHOLD) sweep(now);

      let entry = windows.get(key);
      if (!entry || entry.resetAt <= now) {
        entry = { count: 0, resetAt: now + windowMs };
        windows.set(key, entry);
      }

      entry.count += 1;
      const allowed = entry.count <= limit;

      return {
        allowed,
        limit,
        remaining: Math.max(0, limit - entry.count),
        resetAt: entry.resetAt,
        retryAfterSeconds: Math.max(1, Math.ceil((entry.resetAt - now) / 1000)),
      };
    },
    reset() {
      windows.clear();
    },
  };
}

/** Best-effort client IP from proxy headers; falls back to a shared bucket. */
export function getClientIp(request: Request): string {
  const forwardedFor = request.headers.get("x-forwarded-for");
  if (forwardedFor) {
    const first = forwardedFor.split(",")[0]?.trim();
    if (first) return first;
  }
  return request.headers.get("x-real-ip")?.trim() || "unknown";
}

export function rateLimitHeaders(result: RateLimitResult): Record<string, string> {
  const headers: Record<string, string> = {
    "X-RateLimit-Limit": String(result.limit),
    "X-RateLimit-Remaining": String(result.remaining),
    "X-RateLimit-Reset": String(Math.ceil(result.resetAt / 1000)),
  };
  if (!result.allowed) headers["Retry-After"] = String(result.retryAfterSeconds);
  return headers;
}

function positiveIntFromEnv(name: string, fallback: number): number {
  const parsed = Number.parseInt(process.env[name] ?? "", 10);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : fallback;
}

/**
 * Per-IP limit for POST /api/payments/ticket. Defaults to 5 attempts per
 * minute; tune with TICKET_PURCHASE_RATE_LIMIT / TICKET_PURCHASE_RATE_WINDOW_MS.
 */
export const ticketPurchaseRateLimiter = createRateLimiter({
  limit: positiveIntFromEnv("TICKET_PURCHASE_RATE_LIMIT", 5),
  windowMs: positiveIntFromEnv("TICKET_PURCHASE_RATE_WINDOW_MS", 60_000),
});
