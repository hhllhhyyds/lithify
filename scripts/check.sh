#!/usr/bin/env bash
# Local CI check — run before pushing.
# Usage: ./scripts/check.sh
set -euo pipefail

echo "=== 1/5 fmt ==="
cargo fmt --all -- --check
echo ""

echo "=== 2/5 clippy ==="
cargo clippy --workspace --all-targets -- -D warnings
echo ""

echo "=== 3/5 test ==="
cargo test --workspace
echo ""

echo "=== 4/5 coverage ==="
if command -v cargo-llvm-cov &>/dev/null; then
  cargo llvm-cov test --workspace
else
  echo "cargo-llvm-cov not found — installing..."
  cargo install cargo-llvm-cov
  cargo llvm-cov test --workspace
fi
echo ""

echo "=== 5/5 doc ==="
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
echo ""

echo "All checks passed."
