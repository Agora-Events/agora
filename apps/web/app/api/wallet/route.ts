import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

const STELLAR_HORIZON_URL =
  process.env.STELLAR_HORIZON_URL || "https://horizon-testnet.stellar.org";
const USDC_ISSUER =
  process.env.USDC_ISSUER ||
  "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";
const USDC_ASSET_CODE = "USDC";

/**
 * GET /api/wallet
 *
 * Returns the authenticated user's connected Stellar wallet address,
 * USDC balance fetched from Stellar Horizon, and organizer payout
 * preferences (if the user has an organizer profile).
 *
 * Issue #1042: Settings Page — Payment & Stellar Wallet Section
 */
export const GET = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.sub) {
    throwApiError("Unauthorized", 401);
  }

  const walletAddress = auth!.sub!;

  // Fetch USDC balance from Stellar Horizon
  let usdcBalance = 0;
  try {
    const horizonRes = await fetch(
      `${STELLAR_HORIZON_URL}/accounts/${walletAddress}`,
    );
    if (horizonRes.ok) {
      const account = await horizonRes.json();
      const usdcEntry = (account.balances as Array<{
        asset_type: string;
        asset_code?: string;
        asset_issuer?: string;
        balance: string;
      }>).find(
        (b) =>
          b.asset_type === "credit_alphanum4" &&
          b.asset_code === USDC_ASSET_CODE &&
          b.asset_issuer === USDC_ISSUER,
      );
      if (usdcEntry) {
        usdcBalance = parseFloat(usdcEntry.balance);
      }
    }
  } catch {
    // Balance fetch failing is non-fatal; return 0
    usdcBalance = 0;
  }

  // Fetch organizer payout preferences (optional — only organizers have profiles)
  let payoutPreferences: {
    milestonePlan: string;
    withdrawalCap: number;
  } | null = null;

  try {
    const profile = await prisma.organizerProfile.findUnique({
      where: { address: walletAddress },
      select: { socials: true },
    });

    if (profile) {
      const socials = profile.socials as Record<string, unknown>;
      payoutPreferences = {
        milestonePlan:
          typeof socials?.milestonePlan === "string"
            ? socials.milestonePlan
            : "standard",
        withdrawalCap:
          typeof socials?.withdrawalCap === "number"
            ? socials.withdrawalCap
            : 10000,
      };
    }
  } catch {
    payoutPreferences = null;
  }

  return NextResponse.json({
    wallet: {
      address: walletAddress,
      usdcBalance,
      ...(payoutPreferences ? { payoutPreferences } : {}),
    },
  });
});
