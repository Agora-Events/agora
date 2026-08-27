# Agora Backend API Documentation

This document summarizes the public backend endpoints available.

| Method | Path | Auth | Params/Body | Response | Errors |
|---|---|---|---|---|---|
| POST | `/api/v1/auth/nonce` | None | `{ "address": "G..." }` | `{ "nonce": "hex..." }` | 400 |
| POST | `/api/v1/auth/verify` | None | `{ "address", "nonce", "signature", "public_key" }` | `{ "token": "jwt..." }` + `Set-Cookie: XSRF-TOKEN` | 400, 401 |
| POST | `/api/v1/auth/logout` | None | - | 200 OK | - |
| GET | `/api/v1/profile/me` | Bearer | - | Profile details | 401, 404 |
| GET | `/api/v1/events` | None | `?category=...&sort=...&count=...` | Paginated events with `meta` | 400 |
| GET | `/api/v1/events/map` | None | `?latitude&longitude&radius&limit` | Nearby events with `distance_km` | 400 |
| POST | `/api/v1/events` | Bearer+CSRF | Event JSON | Created event | 400, 401, 403 |
| POST | `/api/v1/tickets/:id/scan` | None | `{ "payload": { ... }, "signature": "...", "public_key": "..." }` | Scan verification result | 400, 403, 404, 409 |

*(This file is maintained manually for quick reference. For a comprehensive machine-readable API, see `/openapi.json`)*

## Error responses

Every error body is a flat JSON object:

```json
{ "code": "NOT_FOUND", "message": "Resource with id '42' was not found" }
```

`code` is a stable machine-readable [`ErrorCode`](src/utils/error.rs). HTTP status
and `message` text are unchanged from previous behaviour; clients should branch
on `code` rather than string-matching `message`.

| Code | HTTP status | When it is returned |
|---|---|---|
| `VALIDATION_FAILED` | 400, 422 | Query/body failed validation (including an unrecognised `?sort=` value) |
| `UNAUTHORIZED` | 401 | Missing or invalid authentication credentials |
| `FORBIDDEN` | 403 | Authenticated caller is not allowed to perform this action |
| `NOT_FOUND` | 404 | The requested resource does not exist |
| `CONFLICT` | 409 | The request conflicts with current resource state (duplicate, unique/FK violation) |
| `RATE_LIMITED` | 429 | The caller exceeded the allowed request rate |
| `INTERNAL_ERROR` | 500 | Unexpected internal failure |
| `SERVICE_UNAVAILABLE` | 502, 503, 504 | Database or downstream service is temporarily unavailable |

## Rate limiting

Sensitive routes are limited to **30 requests per IP per minute**; general routes
to **120 requests per IP per minute**. Every response (allowed or rejected)
includes:

| Header | Meaning |
|---|---|
| `X-RateLimit-Limit` | Maximum requests in the current window |
| `X-RateLimit-Remaining` | Requests remaining in the current window |
| `X-RateLimit-Reset` | Unix timestamp (seconds) when the window refreshes |

A `429` response always includes `Retry-After` set to the seconds remaining in
the current window. The value is an integer ≥ 1.

## Events list

`GET /api/v1/events` returns a cursor-paginated list. Successful `data` includes:

```json
{
  "items": [ /* Event */ ],
  "pagination": { "page_size": 20, "has_more": true, "next_cursor": "..." },
  "meta": { "total": 340, "page_size": 20, "has_more": true }
}
```

- `meta.total` is the COUNT(*) of rows matching the same filters (not just the current page).
- Pass `?count=false` to skip the extra COUNT query; `meta.total` is then omitted.
- `?sort=` accepts `starts_at_asc` (default), `starts_at_desc`, `price_asc`, `price_desc`, `popularity_desc`. Any other value returns `400` with `VALIDATION_FAILED`. Sorting always uses an allow-listed `ORDER BY` plus `id` as a tiebreaker.
