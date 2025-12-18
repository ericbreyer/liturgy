#!/usr/bin/env bash
set -euo pipefail

# release-workspace.sh
# Helper to run cargo-release for the workspace in an idempotent way.
# Usage:
#   ./scripts/release-workspace.sh            # runs cargo release interactively
#   ./scripts/release-workspace.sh --dry-run  # runs cargo release --dry-run
#
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

DRY_RUN=""
if [ "${1-}" = "--dry-run" ]; then
  DRY_RUN="--dry-run"
fi

if ! command -v cargo-release >/dev/null 2>&1 && ! cargo release --version >/dev/null 2>&1; then
  echo "cargo-release is not installed. Install it with:"
  echo "  cargo install cargo-release"
  echo "or run via cargo: cargo release ..."
  exit 1
fi

# Run cargo release in workspace mode. This will bump versions, tag, and publish per release.toml.
CMD=(cargo release --workspace $DRY_RUN)

echo "Running: ${CMD[*]}"
"${CMD[@]}"
