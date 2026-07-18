---
title: 'ADR-011: ASCII art conversion crates MSRV spike — go on figlet-rs + rascii_art'
status: 'accepted'
date: '2026-07-18'
deciders: ['@tiberius']
---

# ADR-011: ASCII art conversion crates MSRV spike — go on figlet-rs + rascii_art

## Status

Accepted (records a GO decision for two crates, NO-GO for a third, and a
deliberate non-adoption of a fourth)

## Context

The 2026-07 strategic improvement plan (`.claude/plans/2026-07-17-research-and-improvement-plan.md`,
Stream C) names ASCII art as a Wave 2 feature, gated by an explicit spike
(C-3, "Task 0 → ADR-011") before the protocol/spec work starts. Four
crates were named as candidates: `figlet-rs` (text → banner), `artem` and
`rascii_art` (image → ASCII, image-family deps are the named risk),
`tui-big-text` (render-time big text — recorded for completeness even
though C-1's design already rejects a render-time approach in favor of
authoring-time pre-rendered `ascii-art` blocks, to keep `fireside-tui`'s
dependency surface exactly as ADR-008 left it).

MSRV is pinned at 1.88 (`Cargo.toml: rust-version = "1.88"`), and the 1.88
toolchain is installed locally. Following the ADR-008 methodology exactly
(a throwaway `cargo init` scratch project, not committed, plus a **real**
`cargo +1.88 build`/`run` — metadata inspection alone was insufficient
last time and is not trusted here either):

- `cargo add figlet-rs artem rascii_art tui-big-text ratatui@0.30` in one
  scratch project resolved 371 packages with **no** `cargo +1.88 build`
  failure — every dependency in the combined tree compiles clean on 1.88.
  Cross-checked with `cargo metadata`: no package in the tree declares a
  `rust-version` greater than 1.88.
- Ran real code, not just a build check: `FIGlet::standard().convert("Fireside")`
  produced correct banner output under `cargo +1.88 run`. `figlet-rs`
  embeds its fonts as resources in the crate (`FIGlet::standard()` etc.
  need no filesystem/network access) and pulls only **11** total packages
  (`cargo tree` in an isolated scratch project) — smallest footprint of
  the four.
- `rascii_art::render_to()` (file-path based) correctly converted a
  synthetic in-memory PNG (16×16 gradient, written via a separate `image
  0.25` in the harness, then read back by rascii_art's own vendored
  `image = "0.24.6"` dependency) into ASCII shading under `cargo +1.88
  run` — confirmed the image decode → grayscale → charset-shading
  pipeline works end-to-end, not just that it compiles. `rascii_art`
  pulls **72** total packages in isolation.
- `artem` also compiled and resolved cleanly on 1.88, but pulls **200**
  total packages in isolation — nearly 3× `rascii_art` for the same
  image→ASCII job. Its default features (`ureq`, `web_image`) add a
  blocking HTTP client (`ureq`, `webpki-roots`, `rustls-webpki`) and a
  zip reader (`zip`) for fetching/loading images from URLs — capability
  Fireside's authoring-time CLI conversion (C-4: local files only, no
  network fetch) has no use for, and would otherwise need
  `default-features = false` gymnastics to strip back down.
- `tui-big-text` also compiled cleanly on 1.88 against `ratatui = "0.30.2"`
  (matching Fireside's pinned version) — recorded per the plan's request,
  not adopted, since C-1's design keeps `fireside-tui` renderer-only for
  ASCII art (reuses the existing spec-005 centered-monospace block path)
  and never depends on a runtime big-text widget.

## Decision

**GO on `figlet-rs` (text → banner) and `rascii_art` (image → ASCII) as
`fireside-cli`-only dependencies. NO-GO on `artem`** — functionally
redundant with `rascii_art` for this feature's needs, at nearly 3× the
dependency weight and with unneeded networking capability. **`tui-big-text`
is not adopted** — C-1's authoring-time design means `fireside-tui` never
needs a big-text rendering widget; this finding exists only so a future
render-time redesign doesn't have to re-spike it from scratch.

Both adopted crates are `fireside-cli`-only (never `fireside-tui`),
consistent with C-4 and this project's existing ADR-006 precedent
(`pulldown-cmark` for `fireside-cli`-only Markdown import) — the
constitution's Principle III allowlist for `fireside-cli` gains
`figlet-rs` and `rascii_art`; the `fireside-tui` row is unchanged, which
is the design's headline property (protocol adds a data-only block kind;
no renderer dependency grows).

## Consequences

### Positive

- No MSRV promise broken by either adopted crate — confirmed by a real
  `cargo +1.88 build`/`run`, not declared metadata alone.
- `rascii_art` over `artem` avoids pulling an HTTP client and zip reader
  into `fireside-cli` for a capability (URL/zip image sources) the
  feature doesn't use, keeping the dependency-allowlist addition as small
  as the job requires.
- `figlet-rs`'s embedded fonts mean zero filesystem/network dependency
  for the text-banner path — no bundling or `include_str!` font asset
  needed beyond what the crate already ships.
- The spike was cheap (~30 minutes of cargo commands, following the
  ADR-008 template) and produced a decisive, empirically-verified answer.

### Negative or Trade-offs

- `rascii_art` vendors its own `image = "0.24.6"` rather than the current
  `image = "0.25"` line; this is invisible to Fireside (no direct `image`
  dependency is added, `fireside-cli` never touches `image` types
  directly — file paths in, `String` ASCII out) but means two `image`
  major-version subtrees can coexist in `Cargo.lock` if any other future
  dependency pulls `image 0.25`. Not a problem today; worth a glance in
  `cargo tree -i image` if it recurs.
- `rascii_art`'s upstream (`UTFeight/RASCII`) is a small, single-purpose
  project; less battle-tested than `figlet-rs`. Acceptable given the
  small, well-defined API surface actually used (`render_to`,
  `RenderOptions`) and that a fork/vendor is cheap if it goes
  unmaintained (single-file core logic).

### Neutral / Follow-up

- Constitution Principle III allowlist amendment: `fireside-cli` row
  gains `figlet-rs`, `rascii_art` (ADR-gated, same pattern as ADR-006).
- Proceed to `/speckit-specify` for spec `009-ascii-art` per the plan;
  cite this ADR and ADR-012 (protocol shape, forthcoming) as prior art.
- If `artem` narrows its default features or a future need for
  URL/zip-sourced images arises, re-spike rather than assume this
  finding still holds — recorded here as it stood on 2026-07-18 against
  `artem 3.0.0`/`rascii_art 0.4.5`.
