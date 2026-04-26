#!/usr/bin/env bash
# Publish the reify-reflect crates to crates.io in dependency order.
# Pass --dry-run to verify packaging without uploading.
#
# Usage:
#   scripts/publish.sh [--dry-run]
#
# Run from the workspace root.

set -euo pipefail

DRY_RUN=""
if [ "${1:-}" = "--dry-run" ]; then
    DRY_RUN="--dry-run"
fi

# Order matters: each crate's path-deps must already be on crates.io
# (or we are using --dry-run, which skips the network resolution step).
CRATES=(
    reify-reflect-core
    reflect-nat
    reflect-derive
    reify-graph
    context-trait
    async-reify-macros
    async-reify
    const-reify
    const-reify-derive
)

for crate in "${CRATES[@]}"; do
    echo "==> publishing ${crate}"
    (cd "${crate}" && cargo publish ${DRY_RUN})
    if [ -z "${DRY_RUN}" ]; then
        # Give crates.io a moment to index before the next crate that depends on it.
        echo "    sleeping 30s to let crates.io index the upload"
        sleep 30
    fi
done

echo "==> publishing facade reify-reflect"
cargo publish ${DRY_RUN}

echo "done"
