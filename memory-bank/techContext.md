# Tech Context

## Core Stack

| Tool                     | Version / Notes                         |
| ------------------------ | --------------------------------------- |
| Rust                     | 2024 edition, MSRV **1.88**             |
| TypeSpec                 | `npm run build` in `models/`            |
| JSON Schema              | 2020-12, generated from TypeSpec        |
| Ratatui                  | TUI framework (`fireside-tui`)          |
| crossterm                | Terminal I/O backend                    |
| syntect + two-face       | Syntax highlighting                     |
| clap                     | CLI argument parsing                    |
| serde + serde_json       | Serialization                           |
| thiserror                | Typed errors in library crates          |
| anyhow                   | Error chains at application boundary    |
| plist                    | iTerm2 `.itermcolors` parsing           |
| font-kit                 | System font discovery                   |
| image                    | Image decoding for node rendering       |
| textwrap                 | Terminal-width text wrapping            |
| tracing                  | Structured logging (warn-level default) |
| Astro 5 + Starlight 0.32 | Documentation site                      |
| cargo-nextest            | Parallel test runner                    |
| cargo-deny               | License + advisory policy               |
| cargo-audit              | Security audit (CI scheduled)           |

## Build & Quality Gate Commands

```bash
# Full local quality gate (run before every commit)
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace          # or: cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Protocol model → JSON Schemas
cd models && npm run build

# Documentation site
cd docs && npm run build              # full static build
cd docs && npm run dev               # dev server at localhost:4321/fireside

# Smoke run
cargo run -- present docs/examples/hello.json
```

## Git Hooks

```bash
# Install once per checkout
bash .githooks/install.sh
```

- `pre-commit`: runs `cargo fmt --check`
- `pre-push`: runs `cargo clippy --workspace -- -D warnings` + nextest

## CI Workflows

| Workflow     | Trigger                                | Jobs                                                           |
| ------------ | -------------------------------------- | -------------------------------------------------------------- |
| `rust.yml`   | push/PR to `main`                      | lint (fmt+clippy+doc), test (ubuntu+macos matrix), msrv (1.88) |
| `docs.yml`   | push/PR to `main`, `docs/**`           | validate + deploy to GitHub Pages                              |
| `models.yml` | push/PR, `models/**` changed           | TypeSpec compile + drift detection                             |
| `audit.yml`  | weekly schedule + `Cargo.lock` changes | cargo-audit + cargo-deny                                       |

## Dependency Policy

- `deny.toml` allowlist: `MIT`, `Apache-2.0`, `BSD-2`, `BSD-3`, `ISC`, `MPL-2.0`, `NCSA`, `Unicode-3.0`.
- Active advisory ignores: `RUSTSEC-2025-0141` (bincode), `RUSTSEC-2024-0436` (paste).

## Protocol Technical Constraints

- JSON wire format uses **kebab-case** for all property names.
- Content block discriminator field is `"kind"` (internal serde tag).
- Extensions use explicit typed form: `{ "kind": "extension", "type": "<reverse.domain.id>", ... }`.
- Enum values in wire format also use kebab-case: `"split-horizontal"`, `"slide-left"`, `"align-right"`, `"code-focus"`.
- `GraphFile` is the serde target; `Graph` is the runtime type (adds `node_index: HashMap<NodeId, usize>`).
- `Graph::from_file` is the only construction path; `Graph::rebuild_index` must be called after any structural mutation.
