# Contributing to Agora

Thanks for helping build Agora! This document is the entry point for contributing to the repository. Area-specific guidance remains in the documents linked below; please read the relevant guide before changing that area.

## Where to start

| Area | Start here | Run from |
|---|---|---|
| Frontend (Next.js, React, Tailwind) | [`apps/web/README.md`](apps/web/README.md) | `apps/web/` |
| Mobile app | [`apps/mobile/README.md`](apps/mobile/README.md) | `apps/mobile/` |
| Rust API server (Axum, PostgreSQL) | [`server/CONTRIBUTING.md`](server/CONTRIBUTING.md) and [`server/README.md`](server/README.md) | `server/` |
| Soroban smart contracts | [`contract/README.md`](contract/README.md) and the [contract guides](contract/contracts/) | `contract/` |
| Testing | [`docs/TESTING.md`](docs/TESTING.md) | Repository root |
| General project setup | [`DEVELOPMENT_SETUP.md`](DEVELOPMENT_SETUP.md) | Repository root |
| Product and architecture documentation | [`docs/`](docs/) | — |

## Prerequisites

Install the following before cloning or building the project:

- [Node.js 24](https://nodejs.org/)
- [pnpm 10](https://pnpm.io/installation) (the repository pins pnpm `10.28.0`)
- [Rust](https://rustup.rs/) with the stable toolchain and `cargo`
- [Docker](https://docs.docker.com/get-docker/) with Docker Compose
- [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) for server migrations
- Soroban CLI when working on contract deployment or network workflows

Verify the core tools:

```bash
node --version
pnpm --version
rustc --version
cargo --version
docker --version
docker compose version
```

Install `sqlx-cli` if needed:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## First-time setup

Clone the repository and install workspace dependencies from the repository root:

```bash
git clone https://github.com/Agora-Events/agora.git
cd agora
pnpm install
```

For backend work, create the local environment file:

```bash
cp server/.env.example server/.env
```

The default local database URL is:

```text
DATABASE_URL=postgres://user:password@localhost:5432/agora
```

Start the local infrastructure with Docker:

```bash
docker compose up -d postgres redis jaeger
```

The compose file provides PostgreSQL on port `5432`, Redis on `6379`, and Jaeger on `16686` (UI), `4317`, and `4318`. Check service status with:

```bash
docker compose ps
```

Run database migrations from the server directory:

```bash
cd server
sqlx migrate run
cd ..
```

You can also start the complete Docker stack with `docker compose up --build`, or use the repository's `make up` shortcut where available. See [`DEVELOPMENT_SETUP.md`](DEVELOPMENT_SETUP.md) for full-stack setup and troubleshooting.

## Run the applications

### Web frontend

```bash
cd apps/web
pnpm dev
```

Open <http://localhost:3000>. Useful commands:

```bash
pnpm lint
pnpm build
pnpm test
pnpm test:ci
pnpm test:visual
```

Some scripts may be available only in the app workspace; use `pnpm run` to list them. For frontend conventions, use [`apps/web/README.md`](apps/web/README.md).

### Rust API server

With PostgreSQL and Redis running, from `server/`:

```bash
cargo run
```

The API listens on <http://localhost:3001> by default. Common checks are documented in [`server/CONTRIBUTING.md`](server/CONTRIBUTING.md):

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

### Mobile app

From `apps/mobile/`:

```bash
pnpm dev
```

Run the same checks used by CI from the repository root:

```bash
pnpm --filter mobile typecheck
pnpm --filter mobile lint
pnpm --filter mobile test
```

Refer to [`apps/mobile/README.md`](apps/mobile/README.md) for platform-specific prerequisites and commands.

### Smart contracts

From `contract/`:

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Contract deployment requires additional Soroban configuration and credentials. Follow [`contract/README.md`](contract/README.md) rather than committing secrets or local environment files.

## Branches and commits

Create branches from the current target branch and use a short, descriptive name:

```bash
git checkout -b feature/your-feature-name
```

Use the existing prefixes in lowercase:

- `feature/` — new functionality
- `fix/` — bug fixes
- `refactor/` — code-only restructuring
- `docs/` — documentation changes

Keep commits focused and use an imperative, Conventional Commit-style subject:

```text
feat: add wallet connection component
fix: resolve token claim validation issue
docs: update contribution guidelines
refactor: modularize payment processing logic
```

Keep the subject concise, explain the motivation in the body when useful, and avoid mixing unrelated changes in one commit.

## Claiming an issue

Before starting, check that the issue is still available and comment that you would like to work on it. Include your ETA. The expected maximum ETA is **48 hours**; after **24 hours**, a finished change or draft PR is expected. Maintainers may unassign an issue when there is no progress or update.

If you become blocked, communicate early by opening a draft PR and tagging the maintainer there. Please use the draft PR for help and progress updates rather than relying on issue comments. Do not start work on an issue that is already assigned without coordinating with the assignee.

## Pull requests

1. Create a branch from the appropriate target branch and keep the change focused.
2. Run the relevant formatters, linters, builds, and tests locally.
3. Open a PR with a clear summary, testing details, and any configuration or migration notes.
4. Link the issue using `Closes #<issue-number>` when the PR completes it.
5. Include screenshots or recordings for UI changes.
6. Call out breaking changes, new dependencies, and follow-up work.
7. Respond to review feedback and keep the branch up to date.

Maintainers review draft PRs within approximately 24 hours when possible. Final PRs are reviewed as soon as possible. A PR can be merged only after required CI checks pass, requested review changes are addressed, and the relevant maintainer approves it.

## CI checks

GitHub Actions runs checks based on the files changed:

- **Frontend CI** (`apps/web/**`): `pnpm --filter web lint` and `pnpm --filter web build`; frontend testing jobs may also run Cypress and Playwright checks. See [`docs/TESTING.md`](docs/TESTING.md).
- **Mobile CI** (`apps/mobile/**`): `pnpm --filter mobile typecheck`, `pnpm --filter mobile lint`, and `pnpm --filter mobile test`.
- **Backend CI** (`server/**`): `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo build --release`.
- **Contract CI** (`contract/**`): formatting, Clippy, and `cargo test --workspace`.

Before requesting review, run the checks relevant to your change and confirm that every required GitHub check is green. Never commit secrets, `.env` files, generated credentials, or unrelated build artifacts.

## Need help?

Read the area-specific guide in the table above, search the existing documentation and issues, and open a draft PR with the current state if you are blocked. Thank you for contributing to Agora!
