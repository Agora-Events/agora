import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError } from "@/lib/api-errors";

const MAX_DISPLAY_NAME = 50;
const MAX_BIO = 500;

/**
 * GET /api/profile
 *
 * Returns the authenticated user's organizer profile.
 * Issue #486: Organizer Profile Management
 */
export const GET = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.sub) {
    throwApiError("Unauthorized", 401);
  }

  const profile = await prisma.organizerProfile.findUnique({
    where: { address: auth!.sub! },
  });

  if (!profile) {
    throwApiError("Profile not found", 404);
  }

  return NextResponse.json({ profile });
});

/**
 * PUT /api/profile
 *
 * Creates or updates the authenticated organizer's profile.
 *
 * Validation:
 * - display_name: required, max 50 chars
 * - bio: optional, max 500 chars
 */
export const PUT = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);
  if (!auth?.sub) {
    throwApiError("Unauthorized", 401);
  }

  let payload: Record<string, unknown>;
  try {
    payload = await request.json();
  } catch {
    throwApiError("Invalid JSON payload", 400);
  }

  const displayName = payload.display_name;
  const bio = payload.bio;
  const avatarUrl = payload.avatar_url;
  const socials = payload.socials ?? {};

  if (typeof displayName !== "string" || displayName.trim().length === 0) {
    throwApiError("display_name is required", 400);
  }

  if (displayName.length > MAX_DISPLAY_NAME) {
    throwApiError(`display_name must be at most ${MAX_DISPLAY_NAME} characters`, 400);
  }

  if (bio !== undefined && bio !== null) {
    if (typeof bio !== "string") {
      throwApiError("bio must be a string", 400);
    }
    if (bio.length > MAX_BIO) {
      throwApiError(`bio must be at most ${MAX_BIO} characters`, 400);
    }
  }

  const profile = await prisma.organizerProfile.upsert({
    where: { address: auth!.sub! },
    create: {
      address: auth!.sub!,
      displayName: (displayName as string).trim(),
      bio: typeof bio === "string" ? bio : null,
      avatarUrl: typeof avatarUrl === "string" ? avatarUrl : null,
      socials: socials as object,
    },
    update: {
      displayName: (displayName as string).trim(),
      bio: typeof bio === "string" ? bio : null,
      avatarUrl: typeof avatarUrl === "string" ? avatarUrl : null,
      socials: socials as object,
    },
  });

  return NextResponse.json({ profile }, { status: 200 });
});
