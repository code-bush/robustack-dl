#!/usr/bin/env bash
# Check that all staged .rs files contain the AI Provenance header.
set -euo pipefail

FAIL=0
for f in "$@"; do
    if ! grep -q "AI PROVENANCE" "$f"; then
        echo "❌ Missing AI Provenance Header: $f"
        FAIL=1
    fi
done
exit "$FAIL"
