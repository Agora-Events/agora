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

## Configuration

The server loads configuration from a `.env` file in this directory. Start from the example file:

```bash
cp .env.example .env
```

PowerShell:

```powershell
Copy-Item .env.example .env
```

> 🚨 **SECURITY WARNING**: *Variables marked with* **[SECRET]** *contain sensitive credentials, API keys, or private tokens. They must* **never** *be committed to version control. Always supply these via a local* `.env` *file or a secure environment secrets manager.*

Required variables:

| Concern | Variable | Required | Default | Description | Example |
|---|---|---|---|---|---|
| **Server / General** | `RUST_ENV` | No | `development` | The application environment. | `production` |
| | `PORT` | Yes | - | The port the server binds to. | `8080` |
| | `RUST_LOG` | No | `info` | The logging level. | `debug` |
| | `BASE_URL` | No | `https://agora.events` | The base URL of the API. | `http://localhost:8080` |
| **Database** | `DATABASE_URL` **[SECRET]** | Yes | - | PostgreSQL connection string. | `postgres://user:pass@localhost/db` |
| | `SLOW_QUERY_THRESHOLD_MS` | Yes | - | Execution time threshold for logging slow DB queries. | `500` |
| **Redis** | `REDIS_URL` | No | `redis://127.0.0.1:6379` | Redis connection string. | `redis://127.0.0.1:6379` |
| **CORS** | `CORS_ALLOWED_ORIGINS` | No | (Code default) | Comma-separated list of allowed CORS origins. | `https://agora.events` |
| **Security & Auth** | `JWT_SECRET` **[SECRET]** | No | `""` | Secret key used for signing JWTs. | `super_secret_string` |
| | `MONITORING_TOKEN` **[SECRET]** | No | `None` | Token used for monitoring endpoints. | `monitor_secret_key` |
| | `ADMIN_TOKEN` **[SECRET]** | No | `None` | Token used for administrative routes. | `admin_secret_key` |
| | `MONITORING_API_KEY` **[SECRET]**| Yes | - | API key required by external monitoring handlers. | `api_key_12345` |
| **Rate Limiting** | `AUTH_RATE_LIMIT_PER_MINUTE` | Yes | - | Rate limit max requests per minute for auth routes. | `10` |
| | `RATE_LIMIT_MAX` | Yes | - | Max requests per rate limit window. | `100` |
| | `RATE_LIMIT_WINDOW` | Yes | - | Time window for rate limiting in seconds. | `60` |
| **Waiting Room** | `WAITING_ROOM_POW_DIFFICULTY`| Yes | - | Proof of Work difficulty setting for the waiting room. | `4` |
| **Indexer / Soroban**| `SOROBAN_RPC_URLS` | No | - | Comma-separated RPC URLs (falls back to `SOROBAN_RPC_URL`). | `https://rpc1...,https://rpc2...` |
| | `SOROBAN_RPC_URL` | Yes | - | Soroban RPC endpoint for the Stellar network. | `https://soroban-testnet.stellar.org` |
| | `TICKET_PAYMENT_CONTRACT_ID` | Yes | - | Stellar Contract ID for ticket payments. | `CA...` |
| | `EVENT_REGISTRY_CONTRACT_ID` | Yes | - | Stellar Contract ID for event registry. | `CB...` |
| | `SOROBAN_START_LEDGER` | Yes | - | The ledger sequence number to start indexing from. | `1000000` |
| | `INDEXER_WINDOW_LEDGERS` | Yes | - | The ledger batch window size for the indexer. | `100` |
| | `INDEXER_CONFIRMATIONS` | Yes | - | Number of network confirmations to wait for. | `1` |
| | `INDEXER_WORKERS` | Yes | - | Number of concurrent worker threads for the indexer. | `4` |
| | `RATES_PROVIDER_URL` | Yes | - | External provider URL for crypto/fiat exchange rates. | `https://api.coingecko.com...` |
| **Notifications** | `EXPO_ACCESS_TOKEN` **[SECRET]** | Yes | - | Access token for Expo push notifications. | `ExponentPushToken[...]` |
| **Storage (S3)** | `S3_BUCKET` | No | `""` | AWS S3 Bucket Name for asset uploads. | `agora-assets` |
| | `S3_REGION` | No | `auto` | AWS S3 Region. | `us-east-1` |
| | `S3_ACCESS_KEY_ID` **[SECRET]** | No | `""` | Access Key ID for S3. | `AKIAIOSFODNN7EXAMPLE` |
| | `S3_SECRET_ACCESS_KEY` **[SECRET]**| No | `""` | Secret Access Key for S3. | `wJalrXUtnFEMI/K7MDENG/b...` |
| | `S3_ENDPOINT_URL` | No | `None` | Custom S3 endpoint URL (used for local/Minio storage). | `http://localhost:9000` |
| | `S3_PUBLIC_URL` | No | `""` | Public CDN/URL prefix for accessing stored assets. | `https://cdn.agora.events` |
| | `ALLOWED_UPLOAD_MIME_TYPES` | No* | - | Primary variable for allowed upload Mime types. | `image/jpeg,image/png` |
| | `ALLOWED_MIME_TYPES` | Yes* | - | Fallback variable if `ALLOWED_UPLOAD_MIME_TYPES` is absent. | `image/jpeg,image/png` |

*(Note: The server checks* `ALLOWED_UPLOAD_MIME_TYPES` *first; if it's missing, it strictly requires* `ALLOWED_MIME_TYPES` *to be set).*

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

## API Documentation

The API is documented in [`openapi.yaml`](./openapi.yaml) (OpenAPI 3.0). It
covers the health probes and event endpoints, including
`GET /api/v1/events/:id/social-proof`, which returns recent purchases,
average rating, waitlist count, and tickets remaining for an event (cached
for 60 seconds, no authentication required).

Lint the spec locally with:

```bash
npx @redocly/cli lint server/openapi.yaml
```

## Pull Request Note

When opening the PR for this issue, include the closing keyword in the PR description:

```text
Closes #issue_number
```
