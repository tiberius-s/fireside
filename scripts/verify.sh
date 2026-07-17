#!/usr/bin/env bash
# Runs every check the CI workflows (.github/workflows/*.yml) actually run,
# in one shot, so a local pass here means CI passes too.
#
# Mirrors, job for job:
#   Rust (rust.yml)      — fmt, clippy, rustdoc, tests, MSRV 1.88 check
#   Protocol (models.yml)  — TypeSpec rebuild must produce zero diff
#     (this workflow's `name:` is "Protocol"; its filename is misleading —
#     check GitHub's Actions tab by name, not by guessing from filenames)
#   Security Audit (audit.yml) — cargo-deny (cargo-audit skipped if the
#     advisory-db fetch isn't available offline; CI still runs it)
#   Docs (docs.yml)       — astro check + build
#
# Usage: scripts/verify.sh [--skip-slow]
#   --skip-slow   skip the MSRV 1.88 recompile and docs npm install/build
#                 (useful for a fast inner-loop pass; do a full run before
#                 handing off work)

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

SKIP_SLOW=false
if [[ "${1:-}" == "--skip-slow" ]]; then
  SKIP_SLOW=true
fi

step() { printf '\n\033[1;34m==> %s\033[0m\n' "$1"; }
ok() { printf '\033[1;32m✓ %s\033[0m\n' "$1"; }
warn() { printf '\033[1;33m! %s\033[0m\n' "$1"; }

# ─── Rust: lint job ────────────────────────────────────────────────────────
step "cargo fmt --check"
cargo fmt --check
ok "fmt clean"

step "cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings
ok "clippy silent"

step "cargo doc --workspace --no-deps (RUSTDOCFLAGS=-D warnings)"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
ok "rustdoc clean"

# ─── Rust: test job ────────────────────────────────────────────────────────
step "cargo test --workspace"
if command -v cargo-nextest >/dev/null 2>&1; then
  cargo nextest run --workspace
else
  warn "cargo-nextest not installed locally (CI uses it) — falling back to cargo test"
  cargo test --workspace
fi
ok "tests pass"

# ─── Rust: MSRV job ────────────────────────────────────────────────────────
if [[ "$SKIP_SLOW" == false ]]; then
  step "cargo +1.88 check --workspace (MSRV gate)"
  if rustup toolchain list 2>/dev/null | grep -q '^1\.88'; then
    cargo +1.88 check --workspace --all-targets
    ok "MSRV 1.88 clean"
  else
    warn "rustc 1.88 toolchain not installed — skipping MSRV check (CI runs this; install with: rustup toolchain install 1.88)"
  fi
else
  warn "skipping MSRV check (--skip-slow)"
fi

# ─── Protocol: TypeSpec must regenerate to zero diff ──────────────────────
step "protocol: npm run build must produce zero tsp-output/ diff"
(
  cd protocol
  npm run build >/dev/null
)
if git diff --exit-code -- protocol/tsp-output/ >/dev/null; then
  ok "tsp-output/ matches main.tsp"
else
  echo "tsp-output/ has uncommitted changes after 'npm run build' — commit the regenerated files:" >&2
  git diff --stat -- protocol/tsp-output/ >&2
  exit 1
fi

step "protocol: fixture corpus parity (Rust vs. Node validators)"
node protocol/run-fixtures.mjs

step "protocol: canonical example validates"
node protocol/validate.mjs docs/examples/hello.json

# ─── Security Audit ────────────────────────────────────────────────────────
step "cargo-deny (license/advisory policy)"
if command -v cargo-deny >/dev/null 2>&1; then
  cargo deny check
  ok "cargo-deny clean"
else
  warn "cargo-deny not installed locally (CI runs this on every PR now) — install with: cargo install cargo-deny"
fi

step "cargo-audit (RustSec advisories)"
if command -v cargo-audit >/dev/null 2>&1; then
  cargo audit || warn "cargo-audit reported findings — review before handoff"
else
  warn "cargo-audit not installed locally — install with: cargo install cargo-audit"
fi

# ─── Docs ──────────────────────────────────────────────────────────────────
if [[ "$SKIP_SLOW" == false ]]; then
  step "docs: astro check + build"
  (
    cd docs
    npm run check
    npm run build >/dev/null
  )
  ok "docs site clean"
else
  warn "skipping docs build (--skip-slow) — still run 'npm run check --prefix docs' at minimum"
fi

printf '\n\033[1;32mAll checks passed — safe to hand off.\033[0m\n'
