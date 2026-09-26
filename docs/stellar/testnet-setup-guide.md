# Stellar Testnet Contributor Setup & Faucet Guide

> **Goal:** Follow this guide from scratch and execute a complete ticket purchase on Stellar Testnet in under 10 minutes.

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

| Variable | Description | Testnet Value |
|---|---|---|
| `NEXT_PUBLIC_STELLAR_NETWORK` | Network identifier consumed by client-side helpers. | `TESTNET` |
| `NEXT_PUBLIC_SOROBAN_RPC_URL` | Soroban RPC endpoint for contract queries. | `https://soroban-testnet.stellar.org` |
| `NEXT_PUBLIC_TICKET_CONTRACT_ID` | Deployed `ticket_payment` contract address. | See [§7](#7-testnet-contract-ids--token-issuers) |
| `NEXT_PUBLIC_TICKET_PAYMENT_CONTRACT_ID` | Same as above — used by the Card Onramp component. | See [§7](#7-testnet-contract-ids--token-issuers) |
| `STELLAR_CONTRACT_ADDRESS` | Server-side contract address for the `mintTicket` util. | See [§7](#7-testnet-contract-ids--token-issuers) |
| `STELLAR_RPC_URL` | Server-side Soroban RPC URL. | `https://soroban-testnet.stellar.org` |
| `STELLAR_NETWORK_PASSPHRASE` | Network passphrase for signing transactions. | `Test SDF Network ; September 2015` |
| `STELLAR_SOURCE_SECRET` | **Server-only.** Secret key of the fee-paying source account. | A funded Testnet `S...` key |

### Auth Variables (also required for full-stack dev)

| Variable | Description |
|---|---|
| `GOOGLE_CLIENT_ID` | Google OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | Google OAuth client secret |
| `JWT_SECRET` | Long random string for session token signing |

> **Security note:** `STELLAR_SOURCE_SECRET` is server-only. Never prefix it with `NEXT_PUBLIC_` and never commit it to version control.

### Minimal `.env.local` for ticket purchase testing

```dotenv
# Stellar / Soroban
NEXT_PUBLIC_STELLAR_NETWORK=TESTNET
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_TICKET_CONTRACT_ID=CCMOCKCONTRACTADDRESS1234567890
NEXT_PUBLIC_TICKET_PAYMENT_CONTRACT_ID=CCMOCKCONTRACTADDRESS1234567890

STELLAR_CONTRACT_ADDRESS=CCMOCKCONTRACTADDRESS1234567890
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
STELLAR_SOURCE_SECRET=S... # your funded Testnet keypair

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
- [ ] `apps/web/.env.local` contains all required `NEXT_PUBLIC_STELLAR_*` variables
- [ ] `pnpm dev` starts without errors

---

## 7. Testnet Contract IDs & Token Issuers

> **Note:** Contract IDs below are reference values for the Agora Testnet deployment. Check `contract/.env.devnet` or ask a maintainer for the currently active addresses if these are stale.

| Name | Address |
|---|---|
| `ticket_payment` contract | `CCMOCKCONTRACTADDRESS1234567890` |
| `event_registry` contract | `CCMOCKREGISTRYADDRESS1234567890` |
| `pro_subscription` contract | `CCMOCKSUBSCRIPTIONADDRESS1234` |
| **USDC issuer (Testnet)** | `GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5` |
| **USDC issuer (Mainnet)** | `GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN` |

### Useful Testnet Links

| Resource | URL |
|---|---|
| Stellar Expert (Testnet explorer) | <https://stellar.expert/explorer/testnet> |
| Friendbot faucet | <https://friendbot.stellar.org> |
| Stellar Laboratory | <https://laboratory.stellar.org/#?network=test> |
| Soroban RPC status | <https://soroban-testnet.stellar.org> |

---

## 8. Common Issues

| Symptom | Likely Cause | Fix |
|---|---|---|
| `TicketModal` shows "Add USDC Trustline" button | Account has no USDC trustline | Click the button and approve in Freighter, or follow §4 |
| Freighter popup shows "Wrong Network" | Freighter is on Mainnet | Switch to Testnet in Freighter settings |
| `Account not found` error in trustline helper | Account not yet funded | Run Friendbot (§3) |
| `op_no_trust` error during purchase | Trustline missing | Add trustline (§4) |
| `op_underfunded` error during purchase | Insufficient USDC balance | Fund USDC via a Testnet faucet or direct transfer |
| Transaction explorer link shows wrong network | `NEXT_PUBLIC_STELLAR_NETWORK` is unset | Set it to `TESTNET` in `.env.local` |
| Repeated RPC calls on every modal open | Cache not warming | Ensure `useTicketAvailability` is called with a stable `eventId`; cache TTL is 30 s |

---

*This guide covers Testnet only. For Mainnet deployment, see [DEPLOYMENT.md](../../apps/web/DEPLOYMENT.md).*
