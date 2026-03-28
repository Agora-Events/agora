# Soroban Project

## Project Structure

This repository uses the recommended structure for a Soroban project:

```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.

## Integration + Coverage

Run the end-to-end integration suite (ticket-payment crate):

```bash
cargo test -p ticket-payment
```

Generate coverage artifacts for the integration suite:

```bash
./scripts/generate_coverage.sh
```

## Devnet Deployment

The project includes a script to deploy the smart contracts to the Soroban devnet (or testnet). This is useful for E2E testing and development.

### 1. Configure Environment

Create a `.env.devnet` file in the `contract/` directory from the template:

```bash
cp .env.devnet.example .env.devnet # Or just edit the existing .env.devnet
```

Update the following variables in `.env.devnet`:
- `SOROBAN_ACCOUNT_SECRET`: Your secret key for deployment.
- `ADMIN_ADDRESS`: The admin address for the contracts.
- `PLATFORM_WALLET`: The address that receives platform fees.
- `USDC_TOKEN_ADDRESS`: The address of the USDC token on devnet (or leave it to deploy a mock token).

### 2. Deploy Contracts

Run the deployment script from the `contract/` directory:

```bash
chmod +x ./scripts/deploy_devnet.sh
./scripts/deploy_devnet.sh
```

The script will:
1. Build all contracts.
2. Deploy the `EventRegistry` and `TicketPayment` contracts.
3. Initialize the contracts with the provided configuration.
4. Output the contract IDs and addresses.

### 3. Upgrade Contracts

To upgrade existing contracts, use the `--upgrade` flag:

```bash
./scripts/deploy_devnet.sh --upgrade
```

Ensure `EVENT_REGISTRY_ID` and `TICKET_PAYMENT_ID` are set in your `.env.devnet` before running an upgrade.
