#!/usr/bin/env bash
# Local CI check — run before pushing.
# Usage: ./scripts/check.sh
set -euo pipefail

echo "=== 1/4 fmt ==="
cargo fmt --all -- --check
echo ""

echo "=== 2/4 clippy ==="
cargo clippy --workspace --all-targets -- -D warnings
echo ""

echo "=== 3/4 test ==="
cargo test --workspace
echo ""

echo "=== 4/4 doc ==="
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
echo ""

echo "All checks passed."
