#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "cargo-llvm-cov is not installed."
  echo "Install with: cargo install cargo-llvm-cov"
  exit 1
fi

mkdir -p coverage

# Focus coverage on the ticket-payment crate where the integration suite lives.
cargo llvm-cov \
  -p ticket-payment \
  --all-features \
  --lcov \
  --output-path coverage/lcov.info

cargo llvm-cov \
  -p ticket-payment \
  --all-features \
  --html \
  --output-dir coverage/html

cargo llvm-cov \
  -p ticket-payment \
  --all-features \
  --summary-only | tee coverage/summary.txt

echo "Coverage artifacts generated:"
echo "  - coverage/lcov.info"
echo "  - coverage/html/index.html"
echo "  - coverage/summary.txt"
