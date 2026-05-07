#!/usr/bin/env bash
set -euo pipefail

# Run real LLM integration tests locally.
# Loads env vars from .env in the project root (if exists), then
# requires ANTHROPIC_API_KEY to be set.
#
# Optional env vars:
#   ANTHROPIC_MODEL         (default: claude-sonnet-4-20250514)
#   ANTHROPIC_MAX_TOKENS    (default: 4096)
#   ANTHROPIC_TIMEOUT_SECS  (default: 60)
#   ANTHROPIC_BASE_URL      (default: https://api.anthropic.com)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

if [ -f "$PROJECT_DIR/.env" ]; then
    set -a
    # shellcheck source=/dev/null
    . "$PROJECT_DIR/.env"
    set +a
fi

if [ -z "${ANTHROPIC_API_KEY:-}" ]; then
    echo "ERROR: ANTHROPIC_API_KEY is not set."
    echo "Create a .env file in the project root with:"
    echo "  export ANTHROPIC_API_KEY=sk-ant-..."
    echo "  export ANTHROPIC_BASE_URL=https://your-proxy.example.com  (optional)"
    exit 1
fi

echo "Running real LLM integration tests..."
echo "  Model: ${ANTHROPIC_MODEL:-claude-sonnet-4-20250514}"
echo "  Base URL: ${ANTHROPIC_BASE_URL:-https://api.anthropic.com}"
cargo test -p lithify-llm -- --ignored --nocapture real_llm
