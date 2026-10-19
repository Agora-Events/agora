import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

export const GET = withErrorHandler(async (_request: NextRequest, { params }) => {
  const resolvedParams = await params;
  const id = resolvedParams.id as string;
  const event = await prisma.event.findUnique({
    where: { id },
  });

  if (!event) {
    throwApiError("Event not found", 404);
  }

  
  const organizerProfile = await prisma.organizerProfile
    .findUnique({ where: { address: event!.organizerWallet } })
    .catch(() => null);

  return NextResponse.json({ event, organizerProfile });
});


