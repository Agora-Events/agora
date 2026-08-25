/**
 * Shared authentication types.
 *
 * Agora uses a custom JWT stored in an HttpOnly `auth_token` cookie (see
 * `lib/auth.ts`), so the browser cannot read the session directly. Client code
 * reads it through `GET /api/auth/session`, which returns the shape below.
 */

/** The authenticated user, as exposed to the browser. */
export interface AuthUser {
  /** Stable identifier: the JWT `sub` when present, otherwise the email. */
  id: string;
  /** Email address on the session token, if the provider supplied one. */
  email: string | null;
  /**
   * Wallet / organizer address for this user. Mirrors the JWT `sub`, which is
   * what `OrganizerProfile.address` is keyed on.
   */
  walletAddress: string | null;
  /** Display name from the organizer profile, if one exists. */
  displayName: string | null;
  /** Avatar URL from the organizer profile, if one exists. */
  avatarUrl: string | null;
  /** Bio from the organizer profile, if one exists. */
  bio: string | null;
}

/** Response body of `GET /api/auth/session`. */
export interface SessionResponse {
  /** `null` when there is no valid session cookie. */
  user: AuthUser | null;
}
