# Agora Backend API Documentation

This document summarizes the public backend endpoints available.

| Method | Path | Auth | Params/Body | Response | Errors |
|---|---|---|---|---|---|
| POST | `/api/v1/auth/nonce` | None | `{ "address": "G..." }` | `{ "nonce": "hex..." }` | 400 |
| POST | `/api/v1/auth/verify` | None | `{ "address", "nonce", "signature", "public_key" }` | `{ "token": "jwt..." }` + `Set-Cookie: XSRF-TOKEN` | 400, 401 |
| POST | `/api/v1/auth/logout` | None | - | 200 OK | - |
| GET | `/api/v1/profile/me` | Bearer | - | Profile details | 401, 404 |
| GET | `/api/v1/events` | None | `?category=...` | List of events | - |
| GET | `/api/v1/events/map` | None | `?latitude&longitude&radius&limit` | Nearby events with `distance_km` | 400 |
| POST | `/api/v1/events` | Bearer+CSRF | Event JSON | Created event | 400, 401, 403 |
| POST | `/api/v1/tickets/:id/scan` | None | `{ "payload": { ... }, "signature": "...", "public_key": "..." }` | Scan verification result | 400, 403, 404, 409 |

*(This file is maintained manually for quick reference. For a comprehensive machine-readable API, see `/openapi.json`)*

## Request validation (Issue #1262)

Request DTOs in `handlers/events.rs`, `handlers/profile.rs` and
`handlers/marketplace.rs` are annotated with `#[serde(deny_unknown_fields)]`.
Posting a JSON body that contains an unrecognised field now returns **`400
Bad Request`** (not `422`) whose message names the offending field, e.g.:

```json
{ "code": 400, "message": "unknown field `organiser_name`, there are remaining ..." }
```

All previously valid payloads continue to succeed.

## Categories caching (Issue #1260)

`GET /api/v1/categories` and `GET /api/v1/categories/:id` now send cache
headers and support conditional requests:

- `Cache-Control: public, max-age=300, stale-while-revalidate=600`
- A weak `ETag` (SHA-256 of the serialised payload) on `200` responses.
- A client that repeats the request with a matching `If-None-Match` header
  receives **`304 Not Modified`** with an empty body.
- Error responses are marked `Cache-Control: no-store` and are never cached.

## Geo coordinate validation (Issue #1259)

`GET /api/v1/events/nearby` and `POST /api/v1/geo/geofences` validate
coordinates at the edge:

- `lat` must be finite and within `[-90, 90]`; `lng` within `[-180, 180]`.
- `NaN` / `inf` inputs are rejected.
- `radius_m` (when supplied) must be `> 0` and `<= 500000` metres (500 km).

Invalid input returns **`400 Bad Request`** naming the offending field, so a
bad request can never surface as a `500` database-constraint error.

## Graceful shutdown (Issue #1261)

The server traps `SIGTERM` and `SIGINT`. On shutdown it:

1. Stops accepting new connections and drains in-flight requests for up to
   `SHUTDOWN_TIMEOUT_SECS` (default `15`).
2. Signals background tasks (Soroban indexer, waiting-room admission worker,
   nonce cleanup) to stop via a cancellation token.
3. Closes the SQLx pool and Redis connection explicitly and logs
   `shutdown complete` with the elapsed duration before exiting `0`.

Set `SHUTDOWN_TIMEOUT_SECS` in the environment to tune the drain window.

