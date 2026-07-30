import { NextResponse } from "next/server";
import { cookies } from "next/headers";

/**
 * POST /api/auth/signout
 *
 * Clears the `auth_token` cookie. Because the cookie is HttpOnly the browser
 * cannot expire it itself, so `useAuth().signOut()` calls this first and then
 * redirects to `/`.
 */
export async function POST() {
  const cookieStore = await cookies();

  cookieStore.set("auth_token", "", {
    httpOnly: true,
    secure: process.env.NODE_ENV === "production",
    sameSite: "lax",
    path: "/",
    maxAge: 0,
  });

  return NextResponse.json({ success: true }, { status: 200 });
}
