import { NextRequest, NextResponse } from "next/server";
import { getAuthFromRequest } from "@/lib/auth";
import { prisma } from "@/lib/prisma";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

export const GET = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.email) {
    throwApiError("Unauthorized", 401);
  }

  const items = await prisma.event.findMany({
    where: { followersOnly: true },
    orderBy: { startsAt: "asc" },
  });

  return NextResponse.json({ items });
});


