import type { NextRequest } from "next/server";
import jwt from "jsonwebtoken";

const JWT_SECRET = process.env.JWT_SECRET || "fallback_secret_for_dev";

type AuthTokenPayload = {
  email?: string;
  sub?: string;
};

export function getAuthFromRequest(request: NextRequest): AuthTokenPayload | null {
  const token = request.cookies.get("auth_token")?.value;

  if (!token) {
    return null;
  }

  try {
    return jwt.verify(token, JWT_SECRET) as AuthTokenPayload;
  } catch {
    return null;
  }
}
