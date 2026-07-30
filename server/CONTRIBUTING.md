# Contributing to Agora Server

Backend is built with **Rust** and **Axum**, using **PostgreSQL** via **sqlx**.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Docker](https://docs.docker.com/get-docker/) (for PostgreSQL)
- [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) for running migrations

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## Environment Setup

1. Copy the example env file:

   ```bash
   cp .env.example .env
   ```

2. Start the database:

   ```bash
   docker compose up -d
   ```

   This starts a PostgreSQL instance at `localhost:5432` with:
   - User: `user` / Password: `password` / DB: `agora`

3. Run migrations:

   ```bash
   sqlx migrate run
   ```

## Development Commands

All commands should be run from the `server/` directory.

| Command | Description |
|---|---|
| `cargo run` | Start the dev server (default port: `3001`) |
| `cargo build` | Compile the project |
| `cargo build --release` | Optimized production build |
| `cargo fmt` | Format code |
| `cargo clippy` | Lint (warnings treated as errors in CI) |
| `cargo test` | Run all tests |

## Database Migrations

Migrations live in `server/migrations/`. To add a new migration:

```bash
sqlx migrate add <migration_name>
```

This creates a new `.sql` file in `migrations/`. Write your SQL, then apply it:

```bash
sqlx migrate run
```

To revert the latest migration:

```bash
sqlx migrate revert
```

## Project Structure

```
src/
├── main.rs       # Server bootstrap
├── lib.rs        # Module exports
├── config/       # Environment & configuration
├── routes/       # Route definitions
├── handlers/     # Request handlers (business logic)
├── models/       # Data models
└── utils/        # Shared helpers
```

## CI Checks

The following must pass before merging (enforced by GitHub Actions):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

Run these locally before opening a PR to avoid CI failures.
