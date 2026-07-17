# Contract: CI workflow changes

No new workflow files. Both changes are edits to existing
`.github/workflows/*.yml` files (see research.md §8 for the full
reasoning).

## `audit.yml` — add `pull_request` trigger

Current `on:` block only has `push` (branch `main`) and a weekly
`schedule`. Add a `pull_request` trigger to the `deny` job's `on:` scope
(or the whole workflow, since both jobs share the file), using the same
`paths` filter already present on the `push` trigger:

```yaml
pull_request:
  paths:
    - 'Cargo.lock'
    - 'deny.toml'
    - '.github/workflows/audit.yml'
```

`cargo audit` (the other job in this file) may stay push/schedule-only if
its advisory-database network dependency makes it a poor PR gate (flagged
for a decision during implementation) — the concrete, spec-required gap is
`cargo deny` not running before merge; `cargo audit`'s trigger scope is a
judgment call, not a requirement.

## `rust.yml` — no change

The existing `msrv` job (`cargo check --workspace` on `dtolnay/rust-toolchain@1.88`)
already runs on every relevant `pull_request` and `push` to `main`
(existing `on:` block, unchanged). This satisfies FR-012 for MSRV; the
decision not to add a dedicated `cargo-msrv` job is recorded in
research.md §8 and MUST be reflected in the plan's progress log / an ADR
note rather than silently assumed.
