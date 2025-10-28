#!/usr/bin/env bash
set -euo pipefail

# publish-crates.sh
# Helper to package (dry-run) or publish workspace crates in dependency order.
# Usage:
#   ./scripts/publish-crates.sh           # dry-run (packages only)
#   ./scripts/publish-crates.sh publish   # actually runs cargo publish (requires credentials)
#
# Order is important: foundational crates (types) must be published first.

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

MODE=${1:-package} # package (default) or publish
DRY_RUN=true
if [ "$MODE" = "publish" ]; then
  DRY_RUN=false
fi

CRATES=(
  "types"
  "cross-proc-cache"
  "calendar_calc"
  "ordo"
  "liturgy-backend"
)

echo "Running in mode: $MODE"

for crate in "${CRATES[@]}"; do
  echo "\n=== Processing $crate ==="

  echo "-- Running tests for $crate"
  cargo test -p "$crate"

  echo "-- Packaging $crate (preview)"
  cargo package -p "$crate" --allow-dirty

  if [ "$DRY_RUN" = false ]; then
    echo "-- Publishing $crate to crates.io"
    # publish will fail if dependency versions are not found on crates.io; publish order matters
    cargo publish -p "$crate"
  else
    echo "-- Dry run: skipping cargo publish for $crate"
  fi

done

echo "All done. If you ran in 'package' mode, verify each package and then run with 'publish' when ready."
