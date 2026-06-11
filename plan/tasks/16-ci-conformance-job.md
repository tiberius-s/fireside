# Task 16 — CI conformance job

**Depends on:** 12, 14
**Crates:** none (CI only)
**Phase:** 3

## Goal

A CI job that fails whenever the JS validator and the Rust CLI disagree about any example or scaffold output — the guardrail that keeps the §1 split-brain from re-opening.

## Steps

1. New `.github/workflows/conformance.yml` triggered on PRs/pushes touching `protocol/**`, `crates/**`, or `docs/examples/**`:
   - checkout; setup Node 20 (cache npm, `protocol/package-lock.json`); `npm ci` in `protocol/`;
   - setup stable Rust + Swatinem cache; `sudo apt-get install -y libfontconfig1-dev` (required by font-kit, same as rust.yml);
   - `cargo build -p fireside-cli`;
   - for every `docs/examples/*.json`: run `node protocol/validate.mjs <f>` and `target/debug/fireside validate <f>`; record exit codes; fail if they differ;
   - scaffold check: `fireside new ci-check --dir /tmp` then both validators on `/tmp/ci-check.json`; both must pass.
2. Implement the comparison as a small shell loop in the workflow (no new scripts directory; keep it in one place).
3. Follow the existing workflow conventions (permissions: `contents: read`, `workflow_dispatch`, names matching audit.yml style).

## Do NOT

- Gate on warnings (only error/exit-code disagreement fails).
- Duplicate rust.yml's test/lint jobs.

## Acceptance

- Workflow YAML is valid (`gh workflow list` after push, or actionlint locally if available).
- Running the loop locally passes:

```bash
for f in docs/examples/*.json; do node protocol/validate.mjs "$f"; cargo run -q -p fireside-cli -- validate "$f"; done
```
