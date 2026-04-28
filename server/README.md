# Agora Server

This directory contains the Rust backend for Agora. The API is built with Axum, uses SQLx for database access and migrations, and stores data in PostgreSQL.

## Tech Stack

### Axum

Axum is the HTTP framework used to define routes, attach middleware layers, manage shared application state, and return typed JSON responses.

### SQLx

SQLx is used for PostgreSQL connectivity, running migrations, and mapping rows into Rust structs. In this codebase, the server creates a `PgPool` at startup and shares it with route handlers through Axum state.

### PostgreSQL

PostgreSQL is the backing database for Agora. The local development database is provided by [`docker-compose.yml`](/c:/Users/User/Desktop/agora/server/docker-compose.yml), and the schema is created from the SQL files in [`migrations/`](/c:/Users/User/Desktop/agora/server/migrations).

## Prerequisites

- Rust stable toolchain
- Cargo
- Docker Desktop or a local PostgreSQL instance
- `sqlx-cli`

Install `sqlx-cli` with PostgreSQL support:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## Environment Variables

These variables are read from `.env` at startup:

| Variable | Required | Default | Purpose |
|---|---|---|---|
| `DATABASE_URL` | Yes | None | PostgreSQL connection string used by the server and SQLx migrations |
| `PORT` | No | `3001` | Port the Axum server binds to |
| `RUST_ENV` | No | `development` | Enables production-specific behavior such as HSTS when set to `production` |
| `RUST_LOG` | No | `info` | Log level for `tracing` output |
| `CORS_ALLOWED_ORIGINS` | No | `http://localhost:3000,http://localhost:5173` | Comma-separated allowlist for browser clients |
| `SOROBAN_RPC_URL` | No | `https://soroban-testnet.stellar.org` | Soroban RPC endpoint used by blockchain connectivity health checks |

Example values are already provided in [`server/.env.example`](/c:/Users/User/Desktop/agora/server/.env.example).

## Local Setup

Run all commands from the [`server/`](/c:/Users/User/Desktop/agora/server) directory.

### 1. Create your local env file

```bash
cp .env.example .env
```

If you are on PowerShell, use:

```powershell
Copy-Item .env.example .env
```

The default local database URL is:

```text
postgres://user:password@localhost:5432/agora
```

### 2. Start PostgreSQL

```bash
docker compose up -d
```

This starts a local PostgreSQL container with:

- Host: `localhost`
- Port: `5432`
- Database: `agora`
- Username: `user`
- Password: `password`

### 3. Run migrations

```bash
sqlx migrate run
```

The migration files live in [`server/migrations/`](/c:/Users/User/Desktop/agora/server/migrations) and create the initial schema for users, organizers, events, ticket tiers, tickets, and transactions.

### 4. Start the server

```bash
cargo run
```

On success, the API will start on `http://localhost:3001` unless you override `PORT`.

### 5. Verify the server

Try the health endpoint:

```bash
curl http://localhost:3001/api/v1/health
```

Try the blockchain health endpoint:

```bash
curl http://localhost:3001/api/v1/health/blockchain
```

## Architecture Overview

The backend follows a straightforward layered Axum structure:

```text
Request
  -> Layer
  -> Route
  -> Handler
  -> Model / Database
  -> Response
```

### Directory Structure

```text
src/
|- main.rs           # Startup: load env, init logging, connect DB, run migrations, serve app
|- lib.rs            # Module exports
|- config/           # Middleware and environment configuration
|- routes/           # Router assembly and endpoint registration
|- handlers/         # HTTP handlers and request/response orchestration
|- models/           # SQLx-backed Rust structs representing database entities
`- utils/            # Shared response, error, logging, and test helpers
```

### Request Lifecycle

1. `main.rs` loads `.env`, initializes logging, reads config, connects to PostgreSQL, and runs embedded SQLx migrations.
2. [`src/routes/mod.rs`](/c:/Users/User/Desktop/agora/server/src/routes/mod.rs) builds the Axum router and applies shared middleware layers.
3. Incoming requests pass through middleware from [`src/config/`](/c:/Users/User/Desktop/agora/server/src/config):
   - request ID generation and propagation
   - CORS configuration
   - security headers
4. The matched route forwards control to a handler in [`src/handlers/`](/c:/Users/User/Desktop/agora/server/src/handlers).
5. The handler uses shared state such as the `PgPool`, performs validation or queries, and builds a response.
6. Model structs in [`src/models/`](/c:/Users/User/Desktop/agora/server/src/models) define the Rust shape of database records and are the right place for table-backed types.
7. Shared helpers in [`src/utils/response.rs`](/c:/Users/User/Desktop/agora/server/src/utils/response.rs) and [`src/utils/error.rs`](/c:/Users/User/Desktop/agora/server/src/utils/error.rs) keep API responses consistent.

### Where To Add New Endpoints

When adding a new API feature, use this flow:

1. Add or update the database schema in [`migrations/`](/c:/Users/User/Desktop/agora/server/migrations) if the feature needs persistence.
2. Add or update a model in [`src/models/`](/c:/Users/User/Desktop/agora/server/src/models) if the endpoint returns or stores a table-backed entity.
3. Create a handler in [`src/handlers/`](/c:/Users/User/Desktop/agora/server/src/handlers) for the request logic.
4. Register the route in [`src/routes/mod.rs`](/c:/Users/User/Desktop/agora/server/src/routes/mod.rs).
5. Reuse helpers in [`src/utils/`](/c:/Users/User/Desktop/agora/server/src/utils) for standard success and error responses.

For example, a new organizer endpoint would typically mean:

- adding `handlers/organizers.rs`
- exporting that module from `handlers/mod.rs`
- wiring routes in `routes/mod.rs`
- using models from `models/organizer.rs`

## Testing

### Health endpoint smoke test

Start the server first, then run:

```bash
bash ./test_health_endpoints.sh
```

If you are using Git Bash on Windows, run the same command there. The script checks:

- `GET /api/v1/health`
- `GET /api/v1/health/blockchain`
- `GET /api/v1/health/db`
- `GET /api/v1/health/ready`

### Rust tests

Run the full test suite with:

```bash
cargo test
```

On Windows, if your default Rust toolchain is GNU and you hit a `dlltool.exe` error, use the installed MSVC toolchain instead:

```powershell
cargo +stable-x86_64-pc-windows-msvc test
```

Useful additional checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Notes For Pull Requests

When you open your PR for this task, include:

```text
Closes #issue_number
```
