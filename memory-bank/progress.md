# Progress

## What Works (as of 2026-02-20)

### Protocol layer

- TypeSpec model (`models/main.tsp`) defines all 18 types; `npm run build` generates all JSON Schemas cleanly.
- Protocol `0.1.0` is fully spec-documented in 6 normative chapters + 3 appendices.
- Extended fields (`Node.title`, `Node.tags`, `Node.duration`, `Graph.fireside-version`, `Graph.extensions`) are additive and in the schema.

### Rust crates

- `fireside-core`: All `ContentBlock` variants, `Graph`/`GraphFile`, traversal types, `CoreError`. Full serde round-trip test coverage.
- `fireside-engine`: Loader, `validate_graph` with `Diagnostic` severity model, `TraversalEngine` (next/choose/goto/back), `CommandHistory` (undo/redo), `PresentationSession`. Fixture test suite + history invariant tests.
- `fireside-tui`: Full `App` state machine, all 5 `AppMode` transitions, `DesignTokens` / `Breakpoint` / `Spacing` / `NodeTemplate` design system, `render_block` pipeline for all 8 block kinds, iTerm2 scheme import, hot-reload, graph view overlay, help overlay. Smoke test passes.
- `fireside-cli`: All 7 subcommands (`present`, `open`, `edit`, `new`, `validate`, `fonts`, `import-theme`), event loop with frame gating and hot-reload, terminal lifecycle management. CLI e2e tests pass.

### DX & CI

- `cargo nextest run --workspace` is green.
- `cargo clippy --workspace -- -D warnings` is clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` is clean.
- CI: `rust.yml` (lint/test/MSRV 1.88), `docs.yml`, `models.yml`, `audit.yml` all green.
- `deny.toml` enforces license allowlist and advisory ignores.
- Git hooks: pre-commit (`fmt --check`), pre-push (clippy + nextest).

### Documentation

- Docs site builds cleanly at 45 pages (Astro + Starlight).
- Sections: Spec, Schemas, Reference, Guides, Crates deep-dives, Explanation.
- Full crate deep-dive set: `fireside-core`, `fireside-engine`, `fireside-tui` (4 articles), `fireside-cli`.
- Learn Rust with Fireside: 9 pages (overview + 8 chapters).

## What's Left / Known Gaps

- **Protocol `0.2.0` planning** — No timeline set. Candidates: export block types, richer extension API, audio/media blocks.
- **Export formats (HTML/PDF)** — Explicitly deferred to `1.0.0` horizon.
- **WCAG contrast enforcement on import** — `contrast_ratio` exists in `DesignTokens` but no validation warning on theme import.
- **Profiling** — No flamegraph/perf analysis has been done. Render pipeline and `App::update` are candidates.
- **Protocol vocabulary drift** — Occasional legacy terms in UI copy; not blocking but worth a pass.

