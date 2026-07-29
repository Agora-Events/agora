#!/bin/bash
# ==============================================================================
# Local Soroban Smart Contract Deployment & Auto .env Injection Script
# ==============================================================================
# Description: Builds event_registry, ticket_payment, and pro_subscription WASMs,
#              verifies local Soroban RPC node health, deploys contracts to local RPC,
#              and automatically injects resulting contract IDs into server/.env
#              and apps/web/.env.local.
# ==============================================================================

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

SOROBAN_RPC_URL="${SOROBAN_RPC_URL:-http://localhost:8000/soroban/rpc}"
NETWORK_PASSPHRASE="${SOROBAN_NETWORK_PASSPHRASE:-Standalone Network ; February 2017}"
SOURCE_ACCOUNT="${SOROBAN_ACCOUNT_SECRET:-SAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA}"

echo "==> Verifying local Soroban RPC node at $SOROBAN_RPC_URL..."
HEALTH_CHECK=$(curl -s -f -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' "$SOROBAN_RPC_URL" 2>/dev/null || true)

if [ -z "$HEALTH_CHECK" ] || ! echo "$HEALTH_CHECK" | grep -q '"status":"healthy"'; then
    echo "Error: Local Soroban RPC node is not running or unhealthy at $SOROBAN_RPC_URL."
    echo "Please ensure your local Soroban standalone node is running before executing this script."
    exit 1
fi

echo "==> Building contracts..."
cd "$PROJECT_ROOT/contract"
cargo build --target wasm32-unknown-unknown --release

echo "==> Deploying contracts to local network..."
DEPLOY_ARGS="--rpc-url $SOROBAN_RPC_URL --network-passphrase \"$NETWORK_PASSPHRASE\" --source-account $SOURCE_ACCOUNT"

EVENT_REGISTRY_WASM="target/wasm32-unknown-unknown/release/event_registry.wasm"
TICKET_PAYMENT_WASM="target/wasm32-unknown-unknown/release/ticket_payment.wasm"
PRO_SUBSCRIPTION_WASM="target/wasm32-unknown-unknown/release/pro_subscription.wasm"

echo "Deploying Event Registry..."
EVENT_REGISTRY_ID=$(stellar contract deploy $DEPLOY_ARGS --wasm "$EVENT_REGISTRY_WASM" 2>/dev/null || soroban contract deploy $DEPLOY_ARGS --wasm "$EVENT_REGISTRY_WASM")

echo "Deploying Ticket Payment..."
TICKET_PAYMENT_ID=$(stellar contract deploy $DEPLOY_ARGS --wasm "$TICKET_PAYMENT_WASM" 2>/dev/null || soroban contract deploy $DEPLOY_ARGS --wasm "$TICKET_PAYMENT_WASM")

echo "Deploying Pro Subscription..."
PRO_SUBSCRIPTION_ID=$(stellar contract deploy $DEPLOY_ARGS --wasm "$PRO_SUBSCRIPTION_WASM" 2>/dev/null || soroban contract deploy $DEPLOY_ARGS --wasm "$PRO_SUBSCRIPTION_WASM")

echo "=============================================================================="
echo "Deployed Contract IDs:"
echo "EVENT_REGISTRY_CONTRACT_ID: $EVENT_REGISTRY_ID"
echo "TICKET_PAYMENT_CONTRACT_ID: $TICKET_PAYMENT_ID"
echo "PRO_SUBSCRIPTION_CONTRACT_ID: $PRO_SUBSCRIPTION_ID"
echo "=============================================================================="

# Helper function to update or append env var in a file
update_env_var() {
    local file="$1"
    local key="$2"
    local val="$3"

    mkdir -p "$(dirname "$file")"
    touch "$file"

    if grep -q "^${key}=" "$file"; then
        # Replace existing key using sed
        sed -i.bak "s|^${key}=.*|${key}=${val}|" "$file" && rm -f "${file}.bak"
    else
        # Append new key
        echo "${key}=${val}" >> "$file"
    fi
}

echo "==> Injecting contract IDs into server/.env..."
SERVER_ENV="$PROJECT_ROOT/server/.env"
update_env_var "$SERVER_ENV" "EVENT_REGISTRY_CONTRACT_ID" "$EVENT_REGISTRY_ID"
update_env_var "$SERVER_ENV" "TICKET_PAYMENT_CONTRACT_ID" "$TICKET_PAYMENT_ID"
update_env_var "$SERVER_ENV" "PRO_SUBSCRIPTION_CONTRACT_ID" "$PRO_SUBSCRIPTION_ID"

echo "==> Injecting contract IDs into apps/web/.env.local..."
WEB_ENV="$PROJECT_ROOT/apps/web/.env.local"
update_env_var "$WEB_ENV" "EVENT_REGISTRY_CONTRACT_ID" "$EVENT_REGISTRY_ID"
update_env_var "$WEB_ENV" "TICKET_PAYMENT_CONTRACT_ID" "$TICKET_PAYMENT_ID"
update_env_var "$WEB_ENV" "PRO_SUBSCRIPTION_CONTRACT_ID" "$PRO_SUBSCRIPTION_ID"
update_env_var "$WEB_ENV" "NEXT_PUBLIC_EVENT_REGISTRY_CONTRACT_ID" "$EVENT_REGISTRY_ID"
update_env_var "$WEB_ENV" "NEXT_PUBLIC_TICKET_PAYMENT_CONTRACT_ID" "$TICKET_PAYMENT_ID"
update_env_var "$WEB_ENV" "NEXT_PUBLIC_PRO_SUBSCRIPTION_CONTRACT_ID" "$PRO_SUBSCRIPTION_ID"
update_env_var "$WEB_ENV" "STELLAR_CONTRACT_ADDRESS" "$TICKET_PAYMENT_ID"

echo "==> Contract deployment and .env injection complete!"
