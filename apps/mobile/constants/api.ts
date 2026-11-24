/**
 * Single source of truth for building backend API URLs.
 *
 * `EXPO_PUBLIC_API_URL` is documented (see `.env.example` / `docs/MOBILE_SETUP.md`)
 * as the host only, with no trailing slash and no `/api/v1` suffix — the app
 * appends that itself. Every service/hook that talks to the backend should
 * import `apiUrl` (or `API_BASE_URL` for the rare case a bare host is needed,
 * e.g. building a WebSocket URL) from here instead of re-deriving its own
 * constant, so the whole app agrees on what the env var means (issue #1789).
 */

/** Host only, e.g. `http://localhost:8080`. No trailing slash, no `/api/v1`. */
export const API_BASE_URL = (process.env.EXPO_PUBLIC_API_URL ?? 'http://localhost:8080').replace(/\/+$/, '');

const API_VERSION_PREFIX = '/api/v1';

/**
 * Joins the API base URL, the `/api/v1` version prefix, and `path` safely —
 * regardless of whether `path` has a leading slash.
 *
 * ```ts
 * apiUrl('/profile/tickets') // -> 'http://localhost:8080/api/v1/profile/tickets'
 * apiUrl('profile/tickets')  // -> 'http://localhost:8080/api/v1/profile/tickets'
 * ```
 */
export function apiUrl(path: string): string {
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${API_BASE_URL}${API_VERSION_PREFIX}${normalizedPath}`;
}
