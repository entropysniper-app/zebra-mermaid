#!/bin/sh
# Runs the differential suites with the given features (default: the default build) and prints
# per-type counts of SVGs identical to Pele. Usage: tools/status.sh [features]
set -eu
cd "$(dirname "$0")/.."
features=${1:-}
if [ -n "$features" ]; then
  args="--no-default-features --features $features"
else
  args=""
fi
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-target/status}
echo "== corpus (Pele documentation examples)"
cargo test -q -p zebra-mermaid $args --test compat_corpus -- --nocapture 2>&1 | grep -E "identical|refused|differ|panicked" || true
echo "== harvested (all inputs from Pele's own tests)"
ZM_FULL_HARVEST=1 cargo test -q -p zebra-mermaid $args --test compat_harvest harvested_full -- --nocapture 2>&1 | grep -E "identical|differ|missing|panicked" || true
