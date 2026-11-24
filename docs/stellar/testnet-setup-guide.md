# Stellar Testnet Contributor Setup & Faucet Guide

> **Goal:** Follow this guide from scratch and execute a complete ticket purchase on Stellar Testnet in under 10 minutes, not counting the one-off contract build and deploy in [§7](#7-testnet-contract-ids--token-issuers).

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Install & Configure Freighter](#2-install--configure-freighter)
3. [Fund Your Account via Friendbot](#3-fund-your-account-via-friendbot)
4. [Create a USDC Trustline](#4-create-a-usdc-trustline)
5. [Environment Variables](#5-environment-variables)
6. [Verify Your Setup](#6-verify-your-setup)
7. [Testnet Contract IDs & Token Issuers](#7-testnet-contract-ids--token-issuers)
8. [Common Issues](#8-common-issues)

---

## 1. Prerequisites

- **Node.js** ≥ 20 (see `.nvmrc` in the repo root)
- **pnpm** ≥ 9 — install with `npm i -g pnpm`
- A Chromium-based browser (Chrome or Brave) — required for the Freighter extension
- The Agora repo cloned and dependencies installed:
  ```bash
  git clone https://github.com/Agora-Events/agora.git
  cd agora
  pnpm install
  ```

---

## 2. Install & Configure Freighter

Freighter is the browser wallet used to sign Stellar transactions on Agora.

### 2.1 Install the Extension

1. Open the [Freighter Chrome Web Store page](https://chrome.google.com/webstore/detail/freighter/bcacfldlkkdogcmkkibnjlakofdplcbk).
2. Click **Add to Chrome** → **Add extension**.
3. Pin the Freighter icon to your browser toolbar for easy access.

### 2.2 Create or Import a Wallet

- On first launch, choose **Create New Wallet** and save your 12-word seed phrase somewhere safe.
- If you already have a Stellar keypair (e.g. from the Stellar Laboratory), choose **Import Wallet** and paste your secret key (`S...`).

### 2.3 Switch to Testnet

Freighter defaults to **Mainnet**. You must switch to Testnet for local development:

1. Click the network selector in the top-left corner of the Freighter popup (it will say **MAINNET**).
2. Select **TESTNET** from the dropdown.
3. Confirm the network label now reads **TESTNET** — your address (displayed as `G...`) is the same across networks, but balances and transactions are isolated.

---

## 3. Fund Your Account via Friendbot

Testnet accounts must be activated before they can hold any asset. The Stellar Foundation provides **Friendbot**, a faucet that seeds 10,000 XLM into a new account.

### 3.1 Get Your Public Key

In Freighter, copy the address shown in the popup (starts with `G`, 56 characters).

### 3.2 Call Friendbot

Replace `<YOUR_PUBLIC_KEY>` below with your copied address:

```bash
curl "https://friendbot.stellar.org?addr=<YOUR_PUBLIC_KEY>"
```

Alternatively, use the [Stellar Laboratory Friendbot UI](https://laboratory.stellar.org/#account-creator?network=test).

A `200 OK` response with a transaction hash confirms the account is funded. You should now see **10,000 XLM** in Freighter.

> **Note:** Friendbot only works on Testnet. Never run it against Mainnet addresses.

---

## 4. Create a USDC Trustline

A **trustline** tells the Stellar network that your account is willing to hold a specific asset. Without a USDC trustline, ticket payments cannot reach your wallet.

Agora handles this automatically in the `TicketModal` UI — if your wallet lacks a trustline, an **"Add USDC Trustline (Testnet)"** button will appear before you reach the purchase confirmation step. Clicking it sends a `ChangeTrust` transaction via Freighter.

### Manual Setup (CLI)

If you prefer to add the trustline before opening the app:

```bash
# Install the Stellar CLI if needed
npm i -g @stellar/stellar-cli

stellar tx new change-trust \
  --asset "USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5" \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

### Via Stellar Laboratory

1. Go to [Stellar Laboratory → Transaction Builder](https://laboratory.stellar.org/#txbuilder?network=test).
2. Set **Source Account** to your public key.
3. Add operation **Change Trust**.
4. Set **Asset** to `USDC` with issuer `GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5`.
5. Sign with your secret key and submit.

> **Testnet USDC Issuer:** `GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5`
>
> This is the canonical Circle-compatible USDC issuer used on Stellar Testnet.

---

## 5. Environment Variables

Copy `apps/web/.env.example` to `apps/web/.env.local` and fill in the values below.

```bash
cp apps/web/.env.example apps/web/.env.local
```

### Required Stellar / Soroban Variables

| Variable                                 | Description                                                         | Testnet Value                                                              |
| ---------------------------------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `NEXT_PUBLIC_STELLAR_NETWORK`            | Network identifier consumed by client-side helpers.                 | `TESTNET`                                                                  |
| `NEXT_PUBLIC_SOROBAN_RPC_URL`            | Soroban RPC endpoint for contract queries.                          | `https://soroban-testnet.stellar.org`                                      |
| `NEXT_PUBLIC_EVENT_CONTRACT_ID`          | Deployed `event_registry` contract ID.                              | **Your own deployment** — see [§7](#7-testnet-contract-ids--token-issuers) |
| `NEXT_PUBLIC_TICKET_CONTRACT_ID`         | Deployed `ticket_payment` contract ID.                              | **Your own deployment** — see [§7](#7-testnet-contract-ids--token-issuers) |
| `NEXT_PUBLIC_TICKET_PAYMENT_CONTRACT_ID` | Same as above — used by the Card Onramp component.                  | **Your own deployment** — see [§7](#7-testnet-contract-ids--token-issuers) |
| `STELLAR_CONTRACT_ADDRESS`               | Server-side `ticket_payment` contract ID for the `mintTicket` util. | **Your own deployment** — see [§7](#7-testnet-contract-ids--token-issuers) |
| `STELLAR_RPC_URL`                        | Server-side Soroban RPC URL.                                        | `https://soroban-testnet.stellar.org`                                      |
| `STELLAR_NETWORK_PASSPHRASE`             | Network passphrase for signing transactions.                        | `Test SDF Network ; September 2015`                                        |
| `STELLAR_SOURCE_SECRET`                  | **Server-only.** Secret key of the fee-paying source account.       | A funded Testnet `S...` key                                                |

> **There are no shared contract IDs to copy.** Agora does not publish a long-lived Testnet deployment, so the contract ID variables have no default value. Deploy your own contracts with [§7](#7-testnet-contract-ids--token-issuers) (the deploy script writes these variables for you). The web app validates every contract ID when it loads `apps/web/lib/stellar/config.ts` and throws if one is missing or is not a valid contract address, so placeholder text will stop it from starting.

### Auth Variables (also required for full-stack dev)

| Variable               | Description                                  |
| ---------------------- | -------------------------------------------- |
| `GOOGLE_CLIENT_ID`     | Google OAuth client ID                       |
| `GOOGLE_CLIENT_SECRET` | Google OAuth client secret                   |
| `JWT_SECRET`           | Long random string for session token signing |

> **Security note:** `STELLAR_SOURCE_SECRET` is server-only. Never prefix it with `NEXT_PUBLIC_` and never commit it to version control.

### Minimal `.env.local` for ticket purchase testing

Every value in angle brackets (`<YOUR_...>`) is a placeholder that you must replace. None of them is a real contract or key. Get the contract IDs by deploying the contracts in [§7](#7-testnet-contract-ids--token-issuers); `./scripts/deploy-testnet.sh` writes the contract ID lines to `apps/web/.env.local` for you.

```dotenv
# Stellar / Soroban
NEXT_PUBLIC_STELLAR_NETWORK=TESTNET
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org

# REPLACE these with the IDs from your own deployment (§7). Placeholders, not real contracts.
NEXT_PUBLIC_EVENT_CONTRACT_ID=<YOUR_EVENT_REGISTRY_CONTRACT_ID>
NEXT_PUBLIC_TICKET_CONTRACT_ID=<YOUR_TICKET_PAYMENT_CONTRACT_ID>
NEXT_PUBLIC_TICKET_PAYMENT_CONTRACT_ID=<YOUR_TICKET_PAYMENT_CONTRACT_ID>

STELLAR_CONTRACT_ADDRESS=<YOUR_TICKET_PAYMENT_CONTRACT_ID>
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
STELLAR_SOURCE_SECRET=<YOUR_FUNDED_TESTNET_SECRET_KEY> # an S... key; never commit it

# Auth (generate JWT secret with: openssl rand -base64 32)
JWT_SECRET=change-me-to-a-long-random-string
NODE_ENV=development
```

---

## 6. Verify Your Setup

Start the development server and confirm everything is wired correctly:

```bash
pnpm dev
```

1. Open [http://localhost:3000](http://localhost:3000).
2. Navigate to any event page.
3. Click **Get Tickets**.
4. The `TicketModal` opens — if Freighter is connected and your wallet has a USDC trustline, you will reach the purchase confirmation step.
5. Confirm the purchase and check the success screen for the **Transaction** link pointing to [stellar.expert/explorer/testnet](https://stellar.expert/explorer/testnet).

### Quick Checklist

- [ ] Freighter installed and set to **TESTNET**
- [ ] Account funded via Friendbot (shows XLM balance in Freighter)
- [ ] USDC trustline added (visible in Freighter's asset list or via Stellar Laboratory)
- [ ] Contracts deployed and initialized on Testnet, and `apps/web/.env.local` contains **your** contract IDs (§7)
- [ ] `apps/web/.env.local` contains all required `NEXT_PUBLIC_STELLAR_*` variables
- [ ] `pnpm dev` starts without errors

---

## 7. Testnet Contract IDs & Token Issuers

> **There are no shared Agora contract IDs for Testnet.** The repository does not maintain a long-lived Testnet deployment, and Stellar periodically resets Testnet, which removes every deployed contract. An ID copied from a document would point at nothing. Deploy your own copies of the contracts and use the IDs you get back.

Contract IDs are 56 characters long and start with `C`. Anything written as `<YOUR_..._CONTRACT_ID>` in this guide is a placeholder for an ID you will have after deploying.

### 7.1 Deploy the contracts

You need:

- The Rust toolchain. The repo pins the version in `rust-toolchain.toml`, and `rustup` installs it on first use, including the `wasm32-unknown-unknown` target.
- The Stellar CLI: `cargo install --locked stellar-cli` (or see the [Stellar CLI setup docs](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup)).

From the repo root:

```bash
./scripts/deploy-testnet.sh
```

The script:

1. Checks that the Stellar CLI is installed and the Testnet RPC is healthy.
2. Uses the secret key in `STELLAR_ACCOUNT_SECRET` as the deployer, or generates an identity called `agora-testnet-deployer` and funds it through Friendbot.
3. Builds the contracts with `cargo build --target wasm32-unknown-unknown --release` in `contract/`. The first build takes several minutes.
4. Deploys `event_registry` and `ticket_payment` with `stellar contract deploy`.
5. Writes the resulting IDs, plus the Testnet RPC URL and passphrase, to `apps/web/.env.local`.
6. Prints each contract ID with a Stellar Expert link.

Optional environment variables:

| Variable                    | Effect                                                                               |
| --------------------------- | ------------------------------------------------------------------------------------ |
| `STELLAR_ACCOUNT_SECRET`    | Deploy from your own funded Testnet account (`S...`) instead of a generated one.     |
| `STELLAR_RPC_URL`           | Use a different Testnet RPC endpoint.                                                |
| `SKIP_BUILD=1`              | Reuse the WASM files from a previous build.                                          |
| `DEPLOY_PRO_SUBSCRIPTION=1` | Also deploy `pro_subscription` and write `NEXT_PUBLIC_PRO_SUBSCRIPTION_CONTRACT_ID`. |

To deploy a single contract by hand instead:

```bash
stellar contract deploy \
  --wasm contract/target/wasm32-unknown-unknown/release/ticket_payment.wasm \
  --source-account <YOUR_DEPLOYER_IDENTITY_OR_SECRET_KEY> \
  --network testnet
```

The command prints the new contract ID.

### 7.2 Initialize and link the contracts

The deploy script only deploys. Each contract must be initialized once, and `event_registry` must be told which `ticket_payment` contract it trusts, before the app can use them. Run these with the deployer account (it becomes the admin). Fill in the first block with your own values; the rest can be pasted as is.

```bash
# --- Replace these placeholders ---------------------------------------------
EVENT_REGISTRY_ID=<YOUR_EVENT_REGISTRY_CONTRACT_ID>
TICKET_PAYMENT_ID=<YOUR_TICKET_PAYMENT_CONTRACT_ID>
SOURCE=agora-testnet-deployer   # the identity name, or your deployer's S... secret key
ADMIN=<YOUR_DEPLOYER_PUBLIC_KEY>   # a G... address; for the generated identity: stellar keys address agora-testnet-deployer
# -----------------------------------------------------------------------------

# Soroban contract for Testnet USDC (derived from the issuer in the table below)
USDC_TOKEN=$(stellar contract id asset \
  --asset USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5 \
  --network testnet)

# 1. Initialize event_registry (platform fee is in basis points: 500 = 5%)
stellar contract invoke --id "$EVENT_REGISTRY_ID" --source-account "$SOURCE" --network testnet \
  -- initialize \
  --admin "$ADMIN" \
  --platform_wallet "$ADMIN" \
  --platform_fee_percent 500 \
  --usdc_token "$USDC_TOKEN"

# 2. Initialize ticket_payment
stellar contract invoke --id "$TICKET_PAYMENT_ID" --source-account "$SOURCE" --network testnet \
  -- initialize \
  --admin "$ADMIN" \
  --usdc_token "$USDC_TOKEN" \
  --platform_wallet "$ADMIN" \
  --event_registry "$EVENT_REGISTRY_ID"

# 3. Link ticket_payment into event_registry
stellar contract invoke --id "$EVENT_REGISTRY_ID" --source-account "$SOURCE" --network testnet \
  -- set_ticket_payment_contract \
  --ticket_payment_address "$TICKET_PAYMENT_ID"
```

If you also deployed `pro_subscription`, initialize it the same way. `pro_monthly_price` is in stroops (7 decimals), so `100000000` is 10 USDC:

```bash
stellar contract invoke --id <YOUR_PRO_SUBSCRIPTION_CONTRACT_ID> --source-account "$SOURCE" --network testnet \
  -- initialize \
  --admin "$ADMIN" \
  --platform_wallet "$ADMIN" \
  --payment_token "$USDC_TOKEN" \
  --pro_monthly_price 100000000
```

> **Note:** If `stellar contract id asset` prints an ID that the explorer cannot find (Testnet was reset and the asset contract was not recreated), deploy it once with `stellar contract asset deploy --asset USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5 --source-account "$SOURCE" --network testnet`. It fails harmlessly if the contract already exists.

### 7.3 Check the result

- Open `https://stellar.expert/explorer/testnet/contract/<YOUR_TICKET_PAYMENT_CONTRACT_ID>` (use your own ID). The page should show the deployed contract and its invocations.
- Run `npx tsx contract/scripts/deploy.ts` to print explorer links for every valid contract ID found in your environment and `.env` files.
- Confirm `apps/web/.env.local` contains your IDs, not the `<YOUR_...>` placeholders. See [§5](#5-environment-variables).

For more detail on the contracts and the other deployment script, see [`contract/README.md`](../../contract/README.md).

### 7.4 Token issuers (real values)

Unlike the contract IDs above, these are real, public network values you can use as they are. They match the defaults in `apps/web/lib/stellar/config.ts`.

| Name                      | Address                                                    |
| ------------------------- | ---------------------------------------------------------- |
| **USDC issuer (Testnet)** | `GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5` |
| **USDC issuer (Mainnet)** | `GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN` |

### Useful Testnet Links

| Resource                          | URL                                             |
| --------------------------------- | ----------------------------------------------- |
| Stellar Expert (Testnet explorer) | <https://stellar.expert/explorer/testnet>       |
| Friendbot faucet                  | <https://friendbot.stellar.org>                 |
| Stellar Laboratory                | <https://laboratory.stellar.org/#?network=test> |
| Soroban RPC status                | <https://soroban-testnet.stellar.org>           |

---

## 8. Common Issues

| Symptom                                                                                                                                                | Likely Cause                                                                                       | Fix                                                                                 |
| ------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `TicketModal` shows "Add USDC Trustline" button                                                                                                        | Account has no USDC trustline                                                                      | Click the button and approve in Freighter, or follow §4                             |
| Freighter popup shows "Wrong Network"                                                                                                                  | Freighter is on Mainnet                                                                            | Switch to Testnet in Freighter settings                                             |
| `Account not found` error in trustline helper                                                                                                          | Account not yet funded                                                                             | Run Friendbot (§3)                                                                  |
| `op_no_trust` error during purchase                                                                                                                    | Trustline missing                                                                                  | Add trustline (§4)                                                                  |
| `op_underfunded` error during purchase                                                                                                                 | Insufficient USDC balance                                                                          | Fund USDC via a Testnet faucet or direct transfer                                   |
| App fails to start with "Invalid event contract ID format", "Invalid ticket contract ID format", or "Event and ticket contract IDs must be configured" | A contract ID variable is missing, still a `<YOUR_...>` placeholder, or is not a valid contract ID | Deploy the contracts (§7) and put your own IDs in `apps/web/.env.local`             |
| Transaction explorer link shows wrong network                                                                                                          | `NEXT_PUBLIC_STELLAR_NETWORK` is unset                                                             | Set it to `TESTNET` in `.env.local`                                                 |
| Repeated RPC calls on every modal open                                                                                                                 | Cache not warming                                                                                  | Ensure `useTicketAvailability` is called with a stable `eventId`; cache TTL is 30 s |

---

_This guide covers Testnet only. For Mainnet deployment, see [DEPLOYMENT.md](../../apps/web/DEPLOYMENT.md)._
