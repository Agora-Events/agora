# PR #1407 — Eager startup validation for all configuration values

## Summary

Closes #1407

Rewrites `server/src/config/mod.rs` to validate **all** configuration values
at server startup rather than at first use. The server now fails fast with a
clear, aggregated error message listing every problem found, instead of
panicking silently mid-request or silently using unsafe fallbacks in
production.

---

## Problem

The previous `Config::from_env()` had several gaps:

| Issue | Example |
|---|---|
| Silent fallback for `JWT_SECRET` | Fell back to a hardcoded dev secret in production |
| Lazy `env::var` calls at use-time | `health.rs` and `soroban_listener.rs` re-read `SOROBAN_RPC_URL` on every request |
| No URL syntax validation | Malformed `DATABASE_URL`, `REDIS_URL`, `BASE_URL` only failed at the first DB/Redis connection |
| Port fallback hid typos | `PORT=abc` silently became 3001 |
| S3 partial config undetected | Setting only `S3_BUCKET` without credentials caused a panic on first upload |
| No environment string validation | `RUST_ENV=superproduction` was accepted silently |
| No CORS URL validation | Invalid or HTTP origins in production were accepted |

---

## Changes

### `server/src/config/mod.rs`
- Added `jwt_secret: String` field to `Config` — no more lazy `env::var("JWT_SECRET")`.
- All validation runs inside `Config::from_env()` before `Ok(Self { … })` is returned.
- Errors are collected into a `Vec<String>` and returned as a single
  `AppError::ValidationError` listing every problem, so operators fix all
  issues in one restart cycle.
- New `s3_enabled()` helper returns `true` only when the full S3 credential
  set is present.

**Validation added:**

| Variable | Rule |
|---|---|
| `DATABASE_URL` | Required; must parse as a URL |
| `PORT` | Must be 1–65535; non-numeric value is an error (no silent fallback) |
| `RUST_ENV` | Must be one of `development`, staging, testing, production |
| `RUST_LOG` | First log-level token must be one of `trace debug info warn error off` |
| `SOROBAN_RPC_URL` | Must parse as a URL |
| `REDIS_URL` | Must parse as a URL |
| `BASE_URL` | Must parse as a URL |
| `S3_ENDPOINT_URL` | Must parse as a URL when present |
| `S3_PUBLIC_URL` | Must parse as a URL when present |
| `CORS_ALLOWED_ORIGINS` | Each comma-separated entry must be a valid URL; in production all must be HTTPS |
| `JWT_SECRET` (production) | Must not be the built-in fallback; must be ≥ 32 chars |
| S3 partial config | All four S3 fields (`S3_BUCKET`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, `S3_PUBLIC_URL`) must be set together or all left unset |
| `S3_ENDPOINT_URL` without `S3_BUCKET` | Rejected as a likely misconfiguration |

### `server/src/handlers/auth.rs`
- Removed the lazy `jwt_secret()` function that called `env::var("JWT_SECRET")`
  on every JWT operation.
- `issue_jwt(address, secret)` and `verify_jwt(token, secret)` now accept the
  secret explicitly.
- `extract_auth(headers, config)` now takes a `&Config` reference — the secret
  comes from the validated startup config.
- Handler signatures for `request_nonce` and `verify_signature` accept
  `Extension(config): Extension<Config>`.

### `server/src/handlers/health.rs`
- `health_check_blockchain` now accepts `Extension(config): Extension<Config>`
  and reads `config.soroban_rpc_url` instead of re-reading `env::var`.

### `server/src/handlers/profile.rs`
- `upsert_profile` and `get_my_profile` updated to accept
  `Extension(config): Extension<Config>` and pass it to `extract_auth`.

### `server/Cargo.toml`
- Added `url = "2.5"` for URL syntax validation in `config/mod.rs`.

### `server/.env.example`
- Documents all new variables (`JWT_SECRET`, `BASE_URL`, full S3 block).
- Notes validation rules inline (port range, required-in-production JWT
  secret length, HTTPS requirement for production CORS origins).

### `.gitignore`
- Added `.kiro/` (Kiro IDE workspace directory).
- Added `pr-*.md` pattern to keep PR notes out of the repository tree.

---

## Testing

All existing config tests are preserved and updated to the new signatures.
New tests cover every validation rule:

- Missing / empty `DATABASE_URL`
- Malformed URL for `DATABASE_URL`, `SOROBAN_RPC_URL`, `REDIS_URL`,
  `BASE_URL`, `S3_ENDPOINT_URL`
- Port 0, port > 65535, non-numeric port
- Invalid `RUST_ENV` string
- Invalid `RUST_LOG` level token
- Invalid CORS origin URL
- HTTP CORS origin in production
- Fallback / short `JWT_SECRET` in production
- Weak `JWT_SECRET` allowed in development
- Partial S3 configuration
- `S3_ENDPOINT_URL` set without `S3_BUCKET`
- Multiple errors reported in a single failure message

Run with:

```bash
cd server
cargo test config
```

---

## Branch

`feat/issue-1407-config-validation`
