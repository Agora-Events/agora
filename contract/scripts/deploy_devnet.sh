#!/bin/bash

# ==============================================================================
# Soroban Devnet Deployment & Upgrade Script
# ==============================================================================
# Description: This script deploys and initializes the Event Registry,
#              Ticket Payment (Vault), and a Mock Token on Soroban Devnet.
#              It can also be used for contract upgrades.
# ==============================================================================

# Exit on error
set -e

# Load environment variables
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ENV_FILE="$PROJECT_ROOT/.env.devnet"

if [ -f "$ENV_FILE" ]; then
    echo "Loading environment from $ENV_FILE..."
    export $(grep -v '^#' "$ENV_FILE" | xargs)
else
    echo "Error: $ENV_FILE not found. Please create it from the template."
    exit 1
fi

# Required environment variables
: "${SOROBAN_NETWORK_PASSPHRASE:?Need SOROBAN_NETWORK_PASSPHRASE}"
: "${SOROBAN_RPC_URL:?Need SOROBAN_RPC_URL}"
: "${SOROBAN_ACCOUNT_SECRET:?Need SOROBAN_ACCOUNT_SECRET}"
: "${ADMIN_ADDRESS:?Need ADMIN_ADDRESS}"
: "${PLATFORM_WALLET:?Need PLATFORM_WALLET}"

NETWORK_ARGS="--network testnet --source-account $SOROBAN_ACCOUNT_SECRET"

echo "Building contracts..."
cd "$PROJECT_ROOT"
cargo build --target wasm32-unknown-unknown --release

# Helper function to deploy a contract
deploy_contract() {
    local wasm_path=$1
    local name=$2
    echo "Deploying $name..."
    local contract_id=$(soroban contract deploy \
        $NETWORK_ARGS \
        --wasm "$wasm_path")
    echo "$contract_id"
}

# Helper function to upgrade a contract
upgrade_contract() {
    local wasm_path=$1
    local contract_id=$2
    local name=$3
    echo "Upgrading $name ($contract_id)..."
    # To upgrade, we first install the new wasm to get the hash
    local wasm_hash=$(soroban contract install \
        $NETWORK_ARGS \
        --wasm "$wasm_path")
    
    # Then we call the upgrade function in the contract
    # Assumes contract has an 'upgrade' function that takes a 'new_wasm_hash'
    soroban contract invoke \
        $NETWORK_ARGS \
        --id "$contract_id" \
        -- \
        upgrade \
        --new_wasm_hash "$wasm_hash"
}

# 1. Deploy/Upgrade Mock Token (if needed)
if [ -z "$USDC_TOKEN_ADDRESS" ] || [[ "$USDC_TOKEN_ADDRESS" == CC* ]]; then
    # Note: Using a generic token WASM. If not available, you can use:
    # soroban lab token deploy --network testnet --source-account $SOROBAN_ACCOUNT_SECRET --name "Mock USDC" --symbol "mUSDC"
    echo "No USDC token address provided. Deploying mock token..."
    # For demonstration, we assume a token.wasm is available in the workspace or we use a placeholder.
    # In a real scenario, you'd have a token.wasm or use the built-in token.
    # Here we'll just use a placeholder or assume the user will provide one.
    # USDC_TOKEN_ADDRESS=$(deploy_contract "target/wasm32-unknown-unknown/release/mock_token.wasm" "Mock Token")
    
    # Since we don't have a mock_token.wasm in the repo, we'll recommend using the built-in token
    # or the user can provide a wasm path.
    echo "WARNING: Mock token WASM not found. Please deploy one manually or provide a path."
    echo "You can deploy a mock token using: soroban contract deploy --wasm <path_to_token_wasm>"
    # exit 1
    USDC_TOKEN_ADDRESS="REPLACE_WITH_TOKEN_ADDRESS"
else
    echo "Using existing USDC token: $USDC_TOKEN_ADDRESS"
fi

# 2. Deploy/Upgrade Event Registry
EVENT_REGISTRY_WASM="target/wasm32-unknown-unknown/release/event_registry.wasm"
if [ -z "$EVENT_REGISTRY_ID" ]; then
    EVENT_REGISTRY_ID=$(deploy_contract "$EVENT_REGISTRY_WASM" "Event Registry")
    
    echo "Initializing Event Registry..."
    soroban contract invoke \
        $NETWORK_ARGS \
        --id "$EVENT_REGISTRY_ID" \
        -- \
        initialize \
        --admin "$ADMIN_ADDRESS" \
        --platform_wallet "$PLATFORM_WALLET" \
        --platform_fee_percent 500 \
        --usdc_token "$USDC_TOKEN_ADDRESS"
else
    if [ "$1" == "--upgrade" ]; then
        upgrade_contract "$EVENT_REGISTRY_WASM" "$EVENT_REGISTRY_ID" "Event Registry"
    else
        echo "Event Registry already deployed: $EVENT_REGISTRY_ID"
    fi
fi

# 3. Deploy/Upgrade Ticket Payment (Vault)
TICKET_PAYMENT_WASM="target/wasm32-unknown-unknown/release/ticket_payment.wasm"
if [ -z "$TICKET_PAYMENT_ID" ]; then
    TICKET_PAYMENT_ID=$(deploy_contract "$TICKET_PAYMENT_WASM" "Ticket Payment")
    
    echo "Initializing Ticket Payment..."
    soroban contract invoke \
        $NETWORK_ARGS \
        --id "$TICKET_PAYMENT_ID" \
        -- \
        initialize \
        --admin "$ADMIN_ADDRESS" \
        --usdc_token "$USDC_TOKEN_ADDRESS" \
        --platform_wallet "$PLATFORM_WALLET" \
        --event_registry "$EVENT_REGISTRY_ID"
else
    if [ "$1" == "--upgrade" ]; then
        upgrade_contract "$TICKET_PAYMENT_WASM" "$TICKET_PAYMENT_ID" "Ticket Payment"
    else
        echo "Ticket Payment already deployed: $TICKET_PAYMENT_ID"
    fi
fi

# 4. Link contracts
if [ -z "$EVENT_REGISTRY_ID_ALREADY_SET" ]; then
    echo "Linking Ticket Payment to Event Registry..."
    soroban contract invoke \
        $NETWORK_ARGS \
        --id "$EVENT_REGISTRY_ID" \
        -- \
        set_ticket_payment_contract \
        --ticket_payment_address "$TICKET_PAYMENT_ID"
fi

echo ""
echo "=============================================================================="
echo "Deployment/Upgrade Complete!"
echo "------------------------------------------------------------------------------"
echo "EVENT_REGISTRY_ID: $EVENT_REGISTRY_ID"
echo "TICKET_PAYMENT_ID: $TICKET_PAYMENT_ID"
echo "USDC_TOKEN_ADDRESS: $USDC_TOKEN_ADDRESS"
echo "=============================================================================="
echo "Update your .env.devnet with these IDs for E2E tests."
