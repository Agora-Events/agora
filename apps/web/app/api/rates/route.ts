import { NextResponse } from "next/server";

/**
 * GET /api/rates
 *
 * Proxies the currency conversion rates from the Rust backend.
 * Falls back to a static USDC=1 USD rate if the backend is unavailable.
 *
 * Issue #495: Add Support for Multiple Currencies in UI
 */
export async function GET() {
  const backendUrl =
    process.env.BACKEND_URL || "http://localhost:3001";

  try {
    const response = await fetch(`${backendUrl}/api/v1/rates`, {
      next: { revalidate: 60 }, // Cache for 60 seconds (matches backend TTL)
    });

    if (!response.ok) {
      throw new Error(`Backend returned ${response.status}`);
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error("Failed to fetch rates from backend:", error);

    // Fallback: return a minimal static response so the UI doesn't break
    return NextResponse.json(
      {
        success: true,
        data: {
          base: "USDC",
          rates: { USD: 1.0 },
          fetched_at: new Date().toISOString(),
        },
        message: "Rates retrieved (fallback)",
      },
      { status: 200 },
    );
  }
}
