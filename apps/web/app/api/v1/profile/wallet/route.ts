import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { withErrorHandler } from "@/lib/api-handler";

/**
 * GET /api/v1/profile/wallet?email=... -> { publicKey }
 *
 * Returns the custodial public key for an email-only (guest) purchaser so
 * the frontend can render ticket QR codes. The encrypted private key is
 * never returned.
 */
export const GET = withErrorHandler(async (request: NextRequest) => {
  const url = request.nextUrl;
  const emailParam = url.searchParams.get("email") || "";
  const email = emailParam.trim().toLowerCase();

  if (!email || !email.includes("@")) {
    return NextResponse.json({ error: "Invalid or missing email" }, { status: 400 });
  }

  const wallet = await prisma.custodialWallet.findUnique({ where: { email } });
  if (!wallet) {
    return NextResponse.json({ error: "Not found" }, { status: 404 });
  }

  return NextResponse.json({ publicKey: wallet.publicKey }, { status: 200 });
});
