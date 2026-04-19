#!/usr/bin/env bash
# Configures git to use the .githooks/ directory for this repository.
# Run once after cloning: .githooks/install.sh

set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

chmod +x "$REPO_ROOT/.githooks/pre-commit"
chmod +x "$REPO_ROOT/.githooks/pre-push"

git -C "$REPO_ROOT" config core.hooksPath .githooks

echo "✅ Git hooks installed. Active hooks:"
echo "   • pre-commit  — fmt check, auto-fix, clippy, docs"
echo "   • pre-push    — fmt check, clippy, docs, pushed-commit checks"
echo ""
echo "   Skip a hook when needed: git push --no-verify"
echo ""
echo "Tip: run the workspace tests manually when you want an end-to-end check."
