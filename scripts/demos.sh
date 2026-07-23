#!/usr/bin/env bash
# Regenerates every VHS demo GIF in .github/ from the release binary.
#
# Usage: scripts/demos.sh [tape-name ...]
#   With no arguments, regenerates all tapes. Pass one or more bare tape
#   names (e.g. `scripts/demos.sh reveal quick-edit`) to regenerate a subset.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if ! command -v vhs >/dev/null 2>&1; then
    echo "error: vhs is not installed (brew install vhs)" >&2
    exit 1
fi

all_tapes=(demo reveal quick-edit editing import validate-watch timer-map art-text art-image)

if [ "$#" -gt 0 ]; then
    tapes=("$@")
else
    tapes=("${all_tapes[@]}")
fi

echo "==> cargo build --release -p fireside-cli"
cargo build --release -p fireside-cli

for name in "${tapes[@]}"; do
    tape=".github/${name}.tape"
    if [ ! -f "$tape" ]; then
        echo "error: no such tape: $tape" >&2
        exit 1
    fi
    echo "==> vhs $tape"
    vhs "$tape"
done
