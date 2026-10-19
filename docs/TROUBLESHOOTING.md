# Troubleshooting & FAQ

This guide collects the most common setup problems contributors hit while working on Agora. Start with the quick fixes below, then use the issue search tips at the end of the file if your error still shows up.

## Quick Fixes

1. Confirm you are using the expected toolchains:
   - Node.js `20+`
   - `pnpm` `10.x` (the repo declares `pnpm@10.28.0` in the root `package.json`)
   - Rust stable from `rustup`
2. Reinstall JavaScript dependencies from the repo root:
   ```bash
   pnpm install
   ```
3. Restart local services after environment changes:
   ```bash
   cd server
   docker compose up -d --force-recreate
   ```
4. Re-run database migrations after pulling schema changes:
   ```bash
   cd server
   sqlx migrate run
   ```
5. Refresh Rust lockfiles and rebuild if Cargo reports stale dependency metadata:
   ```bash
   cd server
   cargo update
   cargo build
   ```
6. Verify Soroban/Stellar environment files before contract or devnet work:
   - `contract/.env.devnet`
   - contract IDs and wallet values required by `contract/scripts/deploy_devnet.sh`

## Database Errors

### Error: `Connection refused`

This usually means PostgreSQL is not running or your app is pointing at the wrong host/port.

Try this:

```bash
cd server
docker compose up -d
```

Then verify the defaults from [`server/CONTRIBUTING.md`](server/CONTRIBUTING.md):

- Host: `localhost`
- Port: `5432`
- User: `user`
- Password: `password`
- Database: `agora`

If you changed `.env` values, restart Docker and the backend after updating them.

### Error: `Schema mismatch`

This usually happens after pulling new migrations without applying them locally.

Quick fix:

```bash
cd server
sqlx migrate run
```

If the error persists:

- Check that your local database is the one your backend is actually using.
- Run `sqlx migrate info` to confirm migration state.
- If your local schema is badly out of sync, recreate the local database before running migrations again.

## Rust Errors

### Toolchain version mismatch

If Cargo, rustfmt, or clippy starts failing unexpectedly, make sure you are on the latest stable toolchain:

```bash
rustup update stable
rustup default stable
rustup show
```

If a stale toolchain is cached in your editor or terminal, restart it after updating Rust.

### Error involving `Cargo.lock`

Common examples:

- lockfile needs to be updated
- package version selection failed
- dependency versions differ between crates

Try this from the crate you are working in:

```bash
cd server
cargo update
cargo build
```

Or for contracts:

```bash
cd contract
cargo update
cargo test
```

Avoid manually editing `Cargo.lock` unless you know exactly why it changed.

## Blockchain Errors

### RPC connection failures

If Soroban or Stellar commands fail to reach RPC services:

- Confirm your internet connection is available.
- Re-check the values in `contract/.env.devnet`.
- Make sure the network target in your command matches the credentials you configured.
- Retry after verifying the RPC URL is reachable from your machine.

For deployment work, re-read [`contract/README.md`](contract/README.md) and confirm the required variables are set before running `./scripts/deploy_devnet.sh`.

### Local network or devnet setup problems

If contract tests or deployment scripts fail because local network state is missing:

- Rebuild the contracts from `contract/`.
- Confirm the Soroban CLI is installed and on your `PATH`.
- Verify `EVENT_REGISTRY_ID` and `TICKET_PAYMENT_ID` are present before running upgrades.
- If you recently changed env values, open a fresh shell so the new values are picked up.

## pnpm / Node Errors

### Node or pnpm version mismatch

The repo uses `pnpm@10.28.0`. If install commands fail or the lockfile looks incompatible:

```bash
node -v
pnpm -v
```

If needed, switch to a supported Node release and reinstall dependencies:

```bash
pnpm install
```

### Missing peer dependencies

If the frontend fails to build because a peer dependency is missing:

- Run `pnpm install` from the repo root, not from a nested package.
- Make sure you did not skip workspace dependencies.
- If the lockfile changed upstream, remove local `node_modules` folders and reinstall.

## Figma Access

### Restricted design links

If the shared Figma file opens with an access request prompt:

- Confirm you are signed into the correct Figma account.
- Request access directly in Figma.
- Add a short note in your draft PR if design access is blocking implementation.
- Use existing screenshots and merged UI patterns in the repo while waiting for access instead of guessing at brand-new layouts.

Useful links:

- [`README.md`](README.md)
- [`contributor.md`](contributor.md)
- [`apps/web/README.md`](apps/web/README.md)

## Search Existing Issues First

Before opening a new issue or asking a maintainer for help, search existing issues for the exact error text.

GitHub search tips:

1. Search the repository issue list with the full error in quotes.
2. Try a narrower search such as:
   - `is:issue is:open "Connection refused"`
   - `is:issue "Schema mismatch"`
   - `is:issue "Cargo.lock"`
   - `is:issue "peer dependency"`
   - `is:issue "RPC"`
3. If nothing useful appears, search closed issues too, because setup fixes are often already documented there.

When you do ask for help, include:

- the full error message
- the command you ran
- whether you are working in `apps/web`, `server`, or `contract`
- what you already tried from this guide
