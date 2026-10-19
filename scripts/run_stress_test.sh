#!/bin/bash
# ==============================================================================
# Agora k6 Stress Test Runner (Issue #1178)
# ==============================================================================
# Thin wrapper around `k6 run scripts/stress_test.js` that:
#   - verifies k6 is installed (with an install hint if not)
#   - waits for the target server's /health endpoint before generating load,
#     so a slow-starting server doesn't get counted as a wall of failures
#   - exports a machine-readable summary for
#     scripts/generate_perf_report.mjs / CI regression reporting
#
# Usage:
#   ./scripts/run_stress_test.sh
#   BASE_URL=http://localhost:3001 VUS=50 DURATION=5m ./scripts/run_stress_test.sh
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

BASE_URL="${BASE_URL:-http://localhost:3001}"
VUS="${VUS:-20}"
DURATION="${DURATION:-2m}"
RAMP_TIME="${RAMP_TIME:-30s}"
OUT_DIR="${OUT_DIR:-$PROJECT_ROOT/scripts/.stress-results}"
WAIT_FOR_SERVER_SECONDS="${WAIT_FOR_SERVER_SECONDS:-60}"

mkdir -p "$OUT_DIR"

if ! command -v k6 >/dev/null 2>&1; then
  echo "ERROR: k6 is not installed." >&2
  echo "  macOS:   brew install k6" >&2
  echo "  Linux:   see https://grafana.com/docs/k6/latest/set-up/install-k6/" >&2
  echo "  CI:      this repo's chaos-bench workflow installs it via grafana/setup-k6-action" >&2
  exit 1
fi

echo "==> Waiting for $BASE_URL/api/v1/health (up to ${WAIT_FOR_SERVER_SECONDS}s)..."
elapsed=0
until curl -fsS "$BASE_URL/api/v1/health" >/dev/null 2>&1; do
  if [ "$elapsed" -ge "$WAIT_FOR_SERVER_SECONDS" ]; then
    echo "ERROR: $BASE_URL did not become healthy within ${WAIT_FOR_SERVER_SECONDS}s" >&2
    exit 1
  fi
  sleep 2
  elapsed=$((elapsed + 2))
done
echo "==> Server is healthy. Starting stress test (VUS=$VUS DURATION=$DURATION RAMP_TIME=$RAMP_TIME)..."

SUMMARY_JSON="$OUT_DIR/summary.json"
SUMMARY_TEXT="$OUT_DIR/summary.txt"

BASE_URL="$BASE_URL" VUS="$VUS" DURATION="$DURATION" RAMP_TIME="$RAMP_TIME" \
  k6 run \
    --summary-export="$SUMMARY_JSON" \
    "$SCRIPT_DIR/stress_test.js" | tee "$SUMMARY_TEXT"

echo ""
echo "==> Stress test complete. Summary written to:"
echo "      $SUMMARY_JSON"
echo "      $SUMMARY_TEXT"
echo "==> Generate a p50/p95/p99 + RPS regression report with:"
echo "      node $SCRIPT_DIR/generate_perf_report.mjs --k6-summary=$SUMMARY_JSON"
