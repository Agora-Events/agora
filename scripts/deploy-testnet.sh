#!/usr/bin/env bash
# ==============================================================================
# Stellar Testnet Contract Deployment & .env Sync  (Issue #1507)
# ==============================================================================
# Builds and deploys event_registry and ticket_payment to Stellar Testnet, then
# writes the resulting contract IDs into apps/web/.env.local and prints Stellar
# Expert verification links.
#
# The testnet counterpart to scripts/deploy-local.sh. Kept as a separate script
# rather than a flag on that one: the local script requires a healthy standalone
# RPC and hardcodes a well-known throwaway secret, while testnet needs Friendbot
# funding, a real key, and explorer links. Threading both through one script
# would mean most of it living under an `if`.
#
# Usage:
#   ./scripts/deploy-testnet.sh
#
# Environment (all optional):
#   STELLAR_ACCOUNT_SECRET  deployer secret (S...). Generated + funded if unset.
#   STELLAR_RPC_URL         default https://soroban-testnet.stellar.org
#   SKIP_BUILD              set to 1 to reuse existing WASM artifacts
#   DEPLOY_PRO_SUBSCRIPTION set to 1 to also deploy pro_subscription
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

STELLAR_RPC_URL="${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org}"
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
FRIENDBOT_URL="https://friendbot.stellar.org"
HORIZON_URL="${HORIZON_URL:-https://horizon-testnet.stellar.org}"
WEB_ENV="$PROJECT_ROOT/apps/web/.env.local"
SKIP_BUILD="${SKIP_BUILD:-0}"

if [[ -t 1 ]]; then
  BOLD=$'\033[1m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; RED=$'\033[31m'; DIM=$'\033[2m'; OFF=$'\033[0m'
else
  BOLD=""; GREEN=""; YELLOW=""; RED=""; DIM=""; OFF=""
fi

step() { printf '%s==>%s %s\n' "$BOLD" "$OFF" "$*"; }
ok()   { printf '%s  ✓%s %s\n' "$GREEN" "$OFF" "$*"; }
warn() { printf '%s  !%s %s\n' "$YELLOW" "$OFF" "$*" >&2; }
note() { printf '%s    %s%s\n' "$DIM" "$*" "$OFF"; }
die()  { printf '%s  ✗%s %s\n' "$RED" "$OFF" "$*" >&2; exit 1; }

# ── Preflight ─────────────────────────────────────────────────────────────────
#
# Both failures the acceptance criteria name are checked here, before anything
# is built — a fifteen-minute cargo build followed by "stellar: command not
# found" is the worst possible ordering.

step "Checking prerequisites"

STELLAR_BIN=""
for candidate in stellar soroban; do
  if command -v "$candidate" >/dev/null 2>&1; then
    STELLAR_BIN="$candidate"
    break
  fi
done

[[ -n "$STELLAR_BIN" ]] || die "$(cat <<'MSG'
Stellar CLI not found.
    Install it with:
      cargo install --locked stellar-cli
    or see https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup
MSG
)"
ok "Using '$STELLAR_BIN' CLI"

command -v curl >/dev/null 2>&1 || die "'curl' is required."

# Reachability is probed with getHealth rather than a bare TCP connect, so a
# reverse proxy answering 200 for a dead backend is still reported as down.
RPC_HEALTH="$(curl -sf --max-time 15 -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' "$STELLAR_RPC_URL" 2>/dev/null || true)"

if [[ -z "$RPC_HEALTH" ]]; then
  die "Testnet RPC unreachable at $STELLAR_RPC_URL. Check the URL or set STELLAR_RPC_URL to a mirror."
elif ! grep -q '"status" *: *"healthy"' <<<"$RPC_HEALTH"; then
  warn "RPC responded but did not report healthy. Continuing; deployment may fail."
  note "$RPC_HEALTH"
else
  ok "Testnet RPC healthy at $STELLAR_RPC_URL"
fi

# ── Deployer identity ─────────────────────────────────────────────────────────

step "Resolving the deployer account"

DEPLOYER_SECRET="${STELLAR_ACCOUNT_SECRET:-}"

if [[ -z "$DEPLOYER_SECRET" ]]; then
  IDENTITY="agora-testnet-deployer"
  note "STELLAR_ACCOUNT_SECRET not set — generating identity '$IDENTITY'"
  "$STELLAR_BIN" keys generate --global "$IDENTITY" --network testnet --overwrite >/dev/null 2>&1 \
    || "$STELLAR_BIN" keys generate "$IDENTITY" --network testnet --overwrite >/dev/null 2>&1 \
    || die "Could not generate a deployer identity."
  DEPLOYER_SECRET="$("$STELLAR_BIN" keys show "$IDENTITY" 2>/dev/null || true)"
  [[ -n "$DEPLOYER_SECRET" ]] || die "Generated an identity but could not read its secret back."
else
  [[ "$DEPLOYER_SECRET" == S* ]] || die "STELLAR_ACCOUNT_SECRET must be a secret key starting with 'S'."
fi

DEPLOYER_ADDRESS="$("$STELLAR_BIN" keys address --secret-key "$DEPLOYER_SECRET" 2>/dev/null \
  || "$STELLAR_BIN" keys address "$DEPLOYER_SECRET" 2>/dev/null || true)"
[[ -n "$DEPLOYER_ADDRESS" ]] || die "Could not derive the public address from the deployer secret."
ok "Deployer: $DEPLOYER_ADDRESS"

# ── Funding ───────────────────────────────────────────────────────────────────

step "Ensuring the deployer is funded"

# Checked before calling Friendbot: it rate-limits, and re-funding an account
# that already exists wastes the quota on a no-op. This is also what makes the
# script safe to re-run.
if curl -sf --max-time 15 "$HORIZON_URL/accounts/$DEPLOYER_ADDRESS" >/dev/null 2>&1; then
  ok "Account already exists on testnet"
else
  note "Account not found — requesting Friendbot funding"
  if curl -sf --max-time 45 "$FRIENDBOT_URL/?addr=$DEPLOYER_ADDRESS" >/dev/null 2>&1; then
    ok "Funded via Friendbot"
  else
    die "Friendbot did not fund $DEPLOYER_ADDRESS. It rate-limits per IP — wait a minute and re-run, or fund the account manually."
  fi
fi

# ── Build ─────────────────────────────────────────────────────────────────────

WASM_DIR="$PROJECT_ROOT/contract/target/wasm32-unknown-unknown/release"

if [[ "$SKIP_BUILD" == "1" ]]; then
  step "Skipping build (SKIP_BUILD=1)"
else
  step "Building contracts for wasm32-unknown-unknown"
  ( cd "$PROJECT_ROOT/contract" && cargo build --target wasm32-unknown-unknown --release )
  ok "Build complete"
fi

# ── Deploy ────────────────────────────────────────────────────────────────────

step "Deploying to Stellar Testnet"

deploy() {
  local wasm_name="$1" label="$2"
  local wasm_path="$WASM_DIR/${wasm_name}.wasm"

  [[ -f "$wasm_path" ]] || die "WASM not found: $wasm_path${SKIP_BUILD:+ (SKIP_BUILD=1 — drop it to build first)}"

  # stderr is kept off stdout so only the contract ID is captured, but it is
  # surfaced on failure — a swallowed CLI error here is the single most
  # confusing way for this script to fail.
  local err_file id
  err_file="$(mktemp)"
  if ! id="$("$STELLAR_BIN" contract deploy \
        --wasm "$wasm_path" \
        --source-account "$DEPLOYER_SECRET" \
        --rpc-url "$STELLAR_RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        2>"$err_file" | tr -d '[:space:]')"; then
    warn "Deployment of $label failed:"
    sed 's/^/      /' "$err_file" >&2
    rm -f "$err_file"
    die "Aborting."
  fi
  rm -f "$err_file"

  # A contract ID is a 56-character strkey beginning with C. Validated because
  # the CLI prints warnings on stdout in some versions, and writing a warning
  # string into .env.local as a contract ID fails much later and much less
  # legibly.
  if [[ ! "$id" =~ ^C[A-Z2-7]{55}$ ]]; then
    warn "Unexpected deploy output for $label: '$id'"
    die "Expected a 56-character contract ID starting with C."
  fi

  printf '%s' "$id"
}

EVENT_CONTRACT_ID="$(deploy event_registry 'Event Registry')"
ok "event_registry  → $EVENT_CONTRACT_ID"

TICKET_CONTRACT_ID="$(deploy ticket_payment 'Ticket Payment')"
ok "ticket_payment  → $TICKET_CONTRACT_ID"

PRO_CONTRACT_ID=""
if [[ "${DEPLOY_PRO_SUBSCRIPTION:-0}" == "1" ]]; then
  PRO_CONTRACT_ID="$(deploy pro_subscription 'Pro Subscription')"
  ok "pro_subscription → $PRO_CONTRACT_ID"
fi

# ── .env sync ─────────────────────────────────────────────────────────────────

step "Writing contract IDs to apps/web/.env.local"

# Rewritten via a temp file rather than in-place sed: the -i flag differs
# between GNU and BSD sed, and this script has to work on both macOS and Linux.
update_env_var() {
  local file="$1" key="$2" val="$3" tmp
  mkdir -p "$(dirname "$file")"
  touch "$file"

  if grep -qE "^${key}=" "$file"; then
    tmp="$(mktemp)"
    grep -vE "^${key}=" "$file" > "$tmp" || true
    printf '%s=%s\n' "$key" "$val" >> "$tmp"
    mv -f "$tmp" "$file"
  else
    printf '%s=%s\n' "$key" "$val" >> "$file"
  fi
}

update_env_var "$WEB_ENV" NEXT_PUBLIC_EVENT_CONTRACT_ID "$EVENT_CONTRACT_ID"
update_env_var "$WEB_ENV" NEXT_PUBLIC_TICKET_CONTRACT_ID "$TICKET_CONTRACT_ID"

# The names deploy-local.sh writes are kept in sync too, so switching between
# local and testnet does not leave the app reading a stale local contract ID
# from a variable this script never touched.
update_env_var "$WEB_ENV" NEXT_PUBLIC_EVENT_REGISTRY_CONTRACT_ID "$EVENT_CONTRACT_ID"
update_env_var "$WEB_ENV" NEXT_PUBLIC_TICKET_PAYMENT_CONTRACT_ID "$TICKET_CONTRACT_ID"
update_env_var "$WEB_ENV" NEXT_PUBLIC_STELLAR_NETWORK testnet
update_env_var "$WEB_ENV" NEXT_PUBLIC_SOROBAN_RPC_URL "$STELLAR_RPC_URL"
update_env_var "$WEB_ENV" NEXT_PUBLIC_STELLAR_NETWORK_PASSPHRASE "$NETWORK_PASSPHRASE"

if [[ -n "$PRO_CONTRACT_ID" ]]; then
  update_env_var "$WEB_ENV" NEXT_PUBLIC_PRO_SUBSCRIPTION_CONTRACT_ID "$PRO_CONTRACT_ID"
fi

ok "Wrote ${WEB_ENV#$PROJECT_ROOT/}"

if ! git -C "$PROJECT_ROOT" check-ignore -q "$WEB_ENV" 2>/dev/null; then
  warn "${WEB_ENV#$PROJECT_ROOT/} is not gitignored — add it before committing."
fi

# ── Summary ───────────────────────────────────────────────────────────────────

EXPLORER="https://stellar.expert/explorer/testnet/contract"

printf '\n%s================================================================%s\n' "$BOLD" "$OFF"
printf '%sDeployed to Stellar Testnet%s\n' "$BOLD" "$OFF"
printf '%s================================================================%s\n\n' "$BOLD" "$OFF"

printf '  event_registry   %s\n' "$EVENT_CONTRACT_ID"
printf '    verify: %s/%s\n\n' "$EXPLORER" "$EVENT_CONTRACT_ID"
printf '  ticket_payment   %s\n' "$TICKET_CONTRACT_ID"
printf '    verify: %s/%s\n\n' "$EXPLORER" "$TICKET_CONTRACT_ID"

if [[ -n "$PRO_CONTRACT_ID" ]]; then
  printf '  pro_subscription %s\n' "$PRO_CONTRACT_ID"
  printf '    verify: %s/%s\n\n' "$EXPLORER" "$PRO_CONTRACT_ID"
fi

printf '  deployer         %s\n' "$DEPLOYER_ADDRESS"
printf '    account: https://stellar.expert/explorer/testnet/account/%s\n\n' "$DEPLOYER_ADDRESS"
printf 'Next: restart the web app so it picks up the new .env.local values.\n\n'
