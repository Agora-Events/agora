# Agora Backend Server

This directory contains the Rust backend for Agora Events. The server exposes a versioned HTTP API with Axum, persists data in PostgreSQL through SQLx, and uses Redis for cache-backed features.

## Tech Stack

- **Axum**: HTTP framework for routing, middleware layers, shared state, and typed responses.
- **SQLx**: Async PostgreSQL access, compile-time friendly query support, connection pooling, and database migrations.
- **PostgreSQL**: Primary relational database for users, organizers, events, tickets, transactions, ratings, audit logs, and related application data.
- **Redis**: Cache layer used by event and rates features. The current server startup requires a reachable Redis instance.

## Prerequisites

- Rust stable toolchain and Cargo
- PostgreSQL 14+ or Docker
- Redis 6+ or Docker
- `sqlx-cli` with PostgreSQL support

Install `sqlx-cli`:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## Environment Variables

The server loads configuration from a `.env` file in this directory. Start from the example file:

```bash
cp .env.example .env
```

PowerShell:

```powershell
Copy-Item .env.example .env
```

Required variables:

| Variable | Example | Description |
| --- | --- | --- |
| `DATABASE_URL` | `postgres://user:password@localhost:5432/agora` | PostgreSQL connection string used by the server and SQLx migrations. |

Optional variables:

| Variable | Default | Description |
| --- | --- | --- |
| `PORT` | `3001` | HTTP port for the Axum server. |
| `RUST_ENV` | `development` | Runtime environment. `production` enables stricter security behavior such as HSTS. |
| `RUST_LOG` | `info` | Log filter used by `tracing-subscriber`. |
| `CORS_ALLOWED_ORIGINS` | `http://localhost:3000,http://localhost:5173` | Comma-separated list of browser origins allowed by CORS. |
| `SOROBAN_RPC_URL` | `https://soroban-testnet.stellar.org` | RPC endpoint used by blockchain health checks. |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL for caching. Startup currently fails if Redis is unavailable. |
| `S3_BUCKET` | empty | Bucket name for image uploads. Required only for upload flows. |
| `S3_REGION` | `auto` | S3/R2 region. `auto` is suitable for Cloudflare R2. |
| `S3_ACCESS_KEY_ID` | empty | S3/R2 access key. Required only for upload flows. |
| `S3_SECRET_ACCESS_KEY` | empty | S3/R2 secret key. Required only for upload flows. |
| `S3_ENDPOINT_URL` | unset | Custom S3/R2 endpoint URL. Required for Cloudflare R2. |
| `S3_PUBLIC_URL` | empty | Public base URL for uploaded files. Required only for upload flows. |

## Local Setup

Run all commands from the `server/` directory.

### 1. Create `.env`

```bash
cp .env.example .env
```

Confirm that `DATABASE_URL` points at your local PostgreSQL database:

```text
DATABASE_URL=postgres://user:password@localhost:5432/agora
```

### 2. Start PostgreSQL

The included Compose file starts PostgreSQL with credentials that match `.env.example`:

```bash
docker compose up -d
```

This creates:

- Host: `localhost`
- Port: `5432`
- Database: `agora`
- Username: `user`
- Password: `password`

If your Docker Compose command is the older standalone binary, use `docker-compose up -d`.

### 3. Start Redis

If Redis is not already running locally, start it with Docker:

```bash
docker run --name agora_redis -p 6379:6379 -d redis:7
```

The default `REDIS_URL` is:

```text
REDIS_URL=redis://127.0.0.1:6379
```

### 4. Run Database Migrations

Apply the SQLx migrations in `migrations/`:

```bash
sqlx migrate run
```

The same migrations are also executed during server startup, but running them explicitly makes setup failures easier to diagnose.

### 5. Run the Server

```bash
cargo run
```

When startup succeeds, the API listens on:

```text
http://localhost:3001
```

Use a different port by setting `PORT` in `.env`.

### 6. Verify Health Endpoints

```bash
curl http://localhost:3001/api/v1/health
curl http://localhost:3001/api/v1/health/db
curl http://localhost:3001/api/v1/health/ready
```

## Architecture Overview

The backend follows a layered Axum architecture:

```text
Request -> Layer -> Route -> Handler -> Model -> Database -> Response
```

### Directory Structure

```text
src/
|-- main.rs            # Loads env, initializes logging, connects services, runs migrations, starts Axum.
|-- lib.rs             # Exposes application modules for the binary and tests.
|-- config/            # Environment config plus CORS, request ID, and security header layers.
|-- routes/            # Builds the Axum Router and registers versioned API paths.
|-- handlers/          # Endpoint functions that validate input, call models/services, and return responses.
|-- models/            # SQLx-backed Rust structs that represent database records and payload shapes.
|-- middleware/        # Request middleware such as audit logging, rate limiting, and request tracing.
|-- cache/             # Redis cache integration.
|-- notifications/     # Email and SMS notification adapters.
`-- utils/             # Shared errors, responses, pagination, logging, and test helpers.
```

### Request Lifecycle

1. `main.rs` loads `.env`, initializes tracing, builds `Config`, opens a `PgPool`, runs SQLx migrations, connects to Redis, and calls `routes::create_routes`.
2. `src/routes/mod.rs` registers API routes under `/api/v1` and applies shared Axum layers.
3. Request layers handle request IDs, tracing, CORS, security headers, rate limits, and route-specific middleware.
4. The matched route calls a handler from `src/handlers`.
5. The handler extracts path/query/body/state values, performs endpoint orchestration, and uses models or shared services for data work.
6. Model types in `src/models` represent database-backed entities and keep SQLx row mapping close to the domain shape.
7. Handlers return consistent API responses through shared utilities in `src/utils`.

### Adding New Endpoints

Use this pattern when adding a new API feature:

1. Add a migration in `migrations/` if the feature needs schema changes.
2. Add or update model types in `src/models/` for database-backed data.
3. Add handler functions in `src/handlers/` for request validation and response construction.
4. Export new handler/model modules from their `mod.rs` files.
5. Register the path in `src/routes/mod.rs`, usually under `/api/v1`.
6. Add route or handler tests for the new behavior.

For example, a new orders API would typically add `src/models/order.rs`, `src/handlers/orders.rs`, export both modules, and nest an `/orders` router from `src/routes/mod.rs`.

## Testing

Run Rust tests:

```bash
cargo test
```

Run formatting and lint checks before opening a PR:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Run the health endpoint smoke test after starting the server:

```bash
bash ./test_health_endpoints.sh
```

The script checks:

- `GET /api/v1/health`
- `GET /api/v1/health/blockchain`
- `GET /api/v1/health/db`
- `GET /api/v1/health/ready`

On Windows, run the script from Git Bash or WSL.

## Pull Request Note

When opening the PR for this issue, include the closing keyword in the PR description:

```text
Closes #issue_number
```
