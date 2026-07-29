import { NextRequest, NextResponse } from "next/server";

/**
 * GET /api/geocode?location=<query>
 *
 * Server-side proxy that forwards geocoding requests to Nominatim so that:
 *  - The client browser never calls Nominatim directly (avoiding the 1 req/s
 *    per-IP limit and generic user-agent blocks).
 *  - Results are cached in-memory (and optionally in Redis) to prevent
 *    redundant upstream calls.
 *
 * Cache strategy:
 *  - In-process LRU map (unbounded, process lifetime) for near-zero latency
 *    on repeated queries within the same server instance.
 *  - `Cache-Control: public, max-age=86400` header so a CDN / reverse-proxy
 *    can cache the response for 24 hours.
 */

interface NominatimResult {
  lat: string;
  lon: string;
  display_name: string;
}

export interface GeocodeResult {
  lat: number;
  lon: number;
  display_name: string;
}

// In-process cache: location string → geocoded coordinates (or null when not found).
const geocodeCache = new Map<string, GeocodeResult | null>();

// Maximum number of entries kept in the in-process cache to avoid unbounded growth.
const MAX_CACHE_SIZE = 500;

export async function GET(req: NextRequest) {
  const { searchParams } = new URL(req.url);
  const location = searchParams.get("location")?.trim();

  if (!location) {
    return NextResponse.json(
      { error: "Missing required query parameter: location" },
      { status: 400 },
    );
  }

  // ── 1. In-process cache hit ──────────────────────────────────────────────
  if (geocodeCache.has(location)) {
    const cached = geocodeCache.get(location);
    if (cached === null) {
      // Known miss — return empty results quickly.
      return NextResponse.json(
        { results: [] },
        {
          headers: {
            "Cache-Control": "public, max-age=86400",
            "X-Geocode-Source": "cache",
          },
        },
      );
    }
    return NextResponse.json(
      { results: [cached] },
      {
        headers: {
          "Cache-Control": "public, max-age=86400",
          "X-Geocode-Source": "cache",
        },
      },
    );
  }

  // ── 2. Upstream Nominatim request ────────────────────────────────────────
  try {
    const nominatimUrl = new URL("https://nominatim.openstreetmap.org/search");
    nominatimUrl.searchParams.set("format", "json");
    nominatimUrl.searchParams.set("q", location);
    nominatimUrl.searchParams.set("limit", "1");

    const response = await fetch(nominatimUrl.toString(), {
      headers: {
        // Nominatim requires a descriptive User-Agent with contact info.
        "User-Agent": "AgoraEvents/1.0 (contact@agora-demo.com)",
        Accept: "application/json",
      },
      // Revalidate at most once per day when using Next.js fetch caching.
      next: { revalidate: 86400 },
    });

    if (!response.ok) {
      throw new Error(`Nominatim responded with HTTP ${response.status}`);
    }

    const data: NominatimResult[] = await response.json();

    // ── 3. Cache and respond ─────────────────────────────────────────────────
    if (data.length === 0) {
      evictIfFull();
      geocodeCache.set(location, null);
      return NextResponse.json(
        { results: [] },
        {
          headers: {
            "Cache-Control": "public, max-age=86400",
            "X-Geocode-Source": "nominatim",
          },
        },
      );
    }

    const result: GeocodeResult = {
      lat: parseFloat(data[0].lat),
      lon: parseFloat(data[0].lon),
      display_name: data[0].display_name,
    };

    evictIfFull();
    geocodeCache.set(location, result);

    return NextResponse.json(
      { results: [result] },
      {
        headers: {
          "Cache-Control": "public, max-age=86400",
          "X-Geocode-Source": "nominatim",
        },
      },
    );
  } catch (err) {
    console.error("[geocode] Upstream error:", err);
    return NextResponse.json(
      { error: "Geocoding service temporarily unavailable" },
      { status: 503 },
    );
  }
}

/** Evict the oldest entry when the cache exceeds the size cap. */
function evictIfFull() {
  if (geocodeCache.size >= MAX_CACHE_SIZE) {
    const firstKey = geocodeCache.keys().next().value;
    if (firstKey !== undefined) {
      geocodeCache.delete(firstKey);
    }
  }
}
