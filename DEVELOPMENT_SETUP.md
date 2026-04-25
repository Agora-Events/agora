# Development Setup Guide

This is the fastest way to go from a fresh clone to a full local Agora development environment.

It covers all three main areas of the repo:

- `apps/web` for the Next.js frontend
- `server` for the Axum + PostgreSQL backend
- `contract` for the Soroban smart contracts

For deeper component-specific details, also see:

- [Root README](./README.md)
- [Frontend README](./apps/web/README.md)
- [Server README](./server/README.md)
- [Contract README](./contract/README.md)

## 1. Prerequisites

Install these tools before starting:

- `Node.js` and `pnpm`
- `Rust` and `cargo`
- `Docker Desktop` or Docker Engine with Compose support
- `sqlx-cli` with PostgreSQL support
- `Soroban CLI`

Recommended checks:

```bash
node --version
pnpm --version
rustc --version
cargo --version
docker --version
docker compose version
sqlx --version
soroban --version
```

Install `sqlx-cli` if you do not already have it:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

If you still need Soroban CLI, install it from the Stellar/Soroban docs before working in `contract/`.

## 2. Clone And Install Workspace Dependencies

From the repo root:

```bash
git clone https://github.com/Agora-Events/agora.git
cd agora
pnpm install
```

This installs the JavaScript dependencies used by the frontend workspace.

## 3. Prepare Environment Files

Before starting services, review these env files:

### Required for local full-stack work

- `server/.env.example` -> create `server/.env`

PowerShell:

```powershell
Copy-Item server/.env.example server/.env
```

Bash:

```bash
cp server/.env.example server/.env
```

Default local backend database value:

```text
DATABASE_URL=postgres://user:password@localhost:5432/agora
```

### Optional unless you are deploying contracts to a network

- `contract/.env.devnet.example` -> create `contract/.env.devnet`

That file is only needed for devnet/testnet deployment flows such as `scripts/deploy_devnet.sh`. It is not required just to run contract tests locally.

### Frontend env status

There is currently no frontend `.env.example` in `apps/web`, so contributors do not need to create an `apps/web/.env.local` file for the default local setup described here.

## 4. Start Infrastructure First

The backend expects PostgreSQL to be running before migrations and server startup.

From `server/`:

```bash
cd server
docker compose up -d
```

This starts PostgreSQL with:

- Host: `localhost`
- Port: `5432`
- Database: `agora`
- Username: `user`
- Password: `password`

To confirm the container is running:

```bash
docker compose ps
```

## 5. Start The Backend

Stay in `server/` and run the database migration first:

```bash
sqlx migrate run
```

Then start the Axum API:

```bash
cargo run
```

The backend should come up on:

```text
http://localhost:3001
```

Notes:

- The server also runs embedded migrations on startup, but contributors should still run `sqlx migrate run` during setup so the database state is explicit.
- Keep this terminal open while working.

## 6. Start The Frontend

Open a new terminal, go to the repo root, and start the Next.js app from `apps/web`:

```bash
cd apps/web
pnpm dev
```

The frontend should come up on:

```text
http://localhost:3000
```

If you have not already installed workspace dependencies from the repo root, do that first with `pnpm install`.

## 7. Run Contract Tests

Open another terminal and run the Soroban contract test suite:

```bash
cd contract
cargo test
```

Useful crate-specific commands:

```bash
cargo test -p event-registry
cargo test -p ticket-payment
```

Use `contract/.env.devnet` only when you are working on contract deployment scripts or remote network deployment flows.

## 8. Recommended Local Execution Order

Use this order for first-time setup:

1. Install prerequisites.
2. Run `pnpm install` from the repo root.
3. Create `server/.env` from `server/.env.example`.
4. Start PostgreSQL with `docker compose up -d` from `server/`.
5. Run `sqlx migrate run` from `server/`.
6. Start the backend with `cargo run` from `server/`.
7. Start the frontend with `pnpm dev` from `apps/web/`.
8. Run `cargo test` from `contract/`.

## 9. Health Verification

Once the backend is running, confirm the system is healthy with these endpoints:

```bash
curl http://localhost:3001/api/v1/health
curl http://localhost:3001/api/v1/health/blockchain
curl http://localhost:3001/api/v1/health/db
curl http://localhost:3001/api/v1/health/ready
```

What to expect:

- `/api/v1/health`: API process is up
- `/api/v1/health/blockchain`: Soroban RPC endpoint is reachable
- `/api/v1/health/db`: database connection is working
- `/api/v1/health/ready`: service is ready to serve requests

You should also be able to open:

- `http://localhost:3000` for the frontend
- `http://localhost:3001/api/v1/health` for the backend

## 10. Troubleshooting

### PostgreSQL will not start

- Make sure Docker Desktop is running.
- Check whether port `5432` is already in use by another local PostgreSQL instance.

### Migrations fail

- Confirm `server/.env` exists.
- Confirm `DATABASE_URL` matches the Docker Compose database settings.
- Make sure the Postgres container is healthy before running `sqlx migrate run`.

### Backend fails to boot

- Verify PostgreSQL is still running with `docker compose ps` in `server/`.
- Recheck `DATABASE_URL`, `PORT`, and `CORS_ALLOWED_ORIGINS` in `server/.env`.

### Frontend fails to start

- Run `pnpm install` from the repo root again.
- Confirm you are starting the app from `apps/web`.

### Contract tests fail immediately

- Confirm Rust and Cargo are installed correctly.
- Confirm Soroban CLI is installed if your contract workflow depends on it.

## 11. PR Reminder

When you open the PR for this task, include the linked issue in the PR description:

```text
Closes #366
```
