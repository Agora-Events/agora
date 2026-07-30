import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { getAuthFromRequest } from "@/lib/auth";
import { withErrorHandler } from "@/lib/api-handler";
import type { AuthUser, SessionResponse } from "@/types/auth";

/**
 * GET /api/auth/session
 *
 * Returns the current session for the browser. The `auth_token` cookie is
 * HttpOnly, so this endpoint is the only way client components can learn who is
 * signed in. Always responds 200 — an absent or invalid token yields
 * `{ user: null }` so callers do not have to treat "signed out" as an error.
 */
export const GET = withErrorHandler(async (request: NextRequest) => {
  const auth = getAuthFromRequest(request);

  if (!auth || (!auth.sub && !auth.email)) {
    return NextResponse.json<SessionResponse>({ user: null });
  }

  const walletAddress = auth.sub ?? null;

  // The profile is optional enrichment: a user can be signed in without having
  // created an organizer profile, and a database hiccup should not sign them out.
  let profile: {
    displayName: string;
    bio: string | null;
    avatarUrl: string | null;
  } | null = null;

  if (walletAddress) {
    try {
      profile = await prisma.organizerProfile.findUnique({
        where: { address: walletAddress },
        select: { displayName: true, bio: true, avatarUrl: true },
      });
    } catch {
      profile = null;
    }
  }

  const user: AuthUser = {
    id: auth.sub ?? auth.email!,
    email: auth.email ?? null,
    walletAddress,
    displayName: profile?.displayName ?? null,
    avatarUrl: profile?.avatarUrl ?? null,
    bio: profile?.bio ?? null,
  };

  return NextResponse.json<SessionResponse>({ user });
});
