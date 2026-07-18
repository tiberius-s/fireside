#!/usr/bin/env bash
# Toggles a valid <-> broken branch target in demo-watch.fireside.json.
# Used by validate-watch.tape to show `fireside validate --watch` catching
# a live edit without embedding JSON quotes in the VHS tape itself.
set -euo pipefail
file=".github/demo-watch.fireside.json"

if grep -q '"target": "finale"' "$file"; then
    sed -i '' 's/"target": "finale"/"target": "missing-node"/' "$file"
else
    sed -i '' 's/"target": "missing-node"/"target": "finale"/' "$file"
fi
