#!/usr/bin/env bash
# Warn if GPG commit signing is not enabled.
set -euo pipefail

SIGN=$(git config --get commit.gpgsign 2>/dev/null || echo "false")
if [ "$SIGN" != "true" ]; then
    echo "⚠️  WARNING: commit.gpgsign is not enabled."
    echo "  Run: git config --local commit.gpgsign true"
    echo "  Commits SHOULD be GPG-signed for provenance integrity."
    exit 1
fi
