import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

export const GET = withErrorHandler(
  async (
    request: NextRequest,
    context: { params: Promise<Record<string, string | string[]>> },
  ) => {
    const { address: rawAddressParam } = await context.params;
    const rawAddress = Array.isArray(rawAddressParam)
      ? rawAddressParam[0]
      : rawAddressParam;
    const auth = getAuthFromRequest(request);

    if (!rawAddress) {
      throwApiError("Profile address is required", 400);
    }

    const address = rawAddress === "me" ? auth?.sub : decodeURIComponent(rawAddress);

    if (!address) {
      throwApiError("Profile address is required", 400);
    }

    const profile = await prisma.organizerProfile.findUnique({
      where: { address },
    });

    if (!profile) {
      throwApiError("Profile not found", 404);
    }

    const [hostedCount, attendedCount] = await Promise.all([
      prisma.event.count({ where: { organizerWallet: address } }),
      prisma.ticket.count({
        where: {
          OR: [{ buyerWallet: address }, { ownerWallet: address }],
        },
      }),
    ]);

    return NextResponse.json({
      profile: {
        ...profile,
        counts: {
          hosted: hostedCount,
          attended: attendedCount,
        },
      },
    });
  },
);
