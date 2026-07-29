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

*(This file is maintained manually for quick reference. For a comprehensive machine-readable API, see `/openapi.json`)*
