# Fireside — Research Findings & Strategic Improvement Plan (2026-07)

> Status: approved 2026-07-17 and handed to a cloud (Ultraplan) session for refinement
> and execution; results land as a PR. This file is the as-approved local version —
> reconcile against the PR if the refined plan diverges.

## Progress Log

_Update this section (don't just rely on git log) whenever a plan item lands
or starts. One line per item: status, date. Checked 2026-07-17: no cloud
(Ultraplan) PR exists yet on `origin` (`gh pr list` empty, no branches
besides `main`) — the work below was done directly in this local session
instead, ahead of/independent from that cloud run. Reconcile if an Ultraplan
PR shows up later._

- [x] B-1 — CI/config correctness fixes — done 2026-07-17 (uncommitted on
      main). `rust.yml`: clippy and the MSRV check now run
      `--all-targets`, so CI actually lints test code (a hard prerequisite
      for A-1/A-2, since both move `#[cfg(test)]` code across files).
      `.cargo/config.toml`: deleted the shadowing `[profile.release]`
      block (`strip = false`) and the invalid `pipelined-compilation` key
      — confirmed via `nm` that the release binary is now actually
      stripped (127 symbols vs. previously unstripped), which it silently
      wasn't before since the config override always won over
      `Cargo.toml`'s `strip = true`. Removed the no-op
      `libfontconfig1-dev` apt installs in all three `rust.yml` jobs
      (confirmed zero `fontconfig` hits in `Cargo.lock`). Bumped
      `actions/checkout@v4`→`@v5` and `actions/setup-node@v4`→`@v5` across
      all four workflow files.
- [x] A-1 — split `render/mod.rs` (1888 → 212 lines) — done 2026-07-17
      (uncommitted on main). Production code moved verbatim into
      `header.rs` (113 lines), `content.rs` (277), `footer.rs` (115),
      `overlays.rs` (135), `hits.rs` (58, `branch_option_hit`/
      `map_row_hit` re-exported so `render::` call sites in `app.rs` are
      unchanged); `mod.rs` kept `draw`/`areas`/`surface`/`Surface`/
      `overlay_rect`/`apply_hyperlinks`/`max_scroll`/the three layout
      consts, all still default-private and thus automatically visible to
      every child module (Rust's "private is visible to descendants"
      rule) — only the reverse direction (child exposing something to the
      parent or a sibling) needed explicit `pub(super)`. One cross-module
      wrinkle: `map.rs` already called `super::indicator` and
      `super::overlay_rect`; `overlay_rect` stayed put so that call was
      untouched, but `indicator` moved into `content.rs` per the plan's
      file assignment, so `mod.rs` re-exports it
      (`use content::indicator;`) to keep `map.rs` byte-identical. All 51
      scenario tests moved verbatim to `render/tests.rs` (`#[cfg(test)]
      mod tests;`), same module path `render::tests::*` as before — no
      snapshot-name churn when A-3 (insta) lands. Verified: `cargo test
      --workspace` (195 passed, same total as before the split),
      `cargo clippy --workspace --all-targets -- -D warnings` clean
      (would have caught unlinted moved test code without B-1), `cargo
      fmt` (only whitespace/import-order diffs from de-nesting), full
      `scripts/verify.sh` green, and a real-terminal tmux smoke of
      `fireside demo` exercising the map screen and the quick-edit modal
      (the two overlays that moved) — both rendered correctly.
      Pre-existing, unrelated to this work: `cargo check`/`clippy` warned
      `unused manifest key: workspace.dev-dependencies` on every run.
      Fixed 2026-07-17: `git log -p` showed `[workspace.dev-dependencies]`
      was never valid — it started life as a plain `[dev-dependencies]`
      table at the workspace root (also not a real Cargo.toml section),
      then got "fixed" by renaming to `workspace.dev-dependencies`, which
      isn't real either. `pretty_assertions` was consequently never
      resolvable and — confirmed via
      `grep -rn pretty_assertions --include='*.rs'` — never imported by
      any crate either, so the honest fix was deleting the dead section
      entirely rather than resurrecting an unused dependency (also not on
      any crate's Principle III allowlist). Verified: the manifest
      warning is gone from `cargo check`/`clippy`, `cargo clippy
      --workspace --all-targets -- -D warnings` and `cargo fmt --check`
      stay clean, all 195 tests still pass.
- [x] A-2 — split `main.rs` (1096 → 297 lines) — done 2026-07-17
      (uncommitted on main). Followed the existing `import.rs`/`resume.rs`
      precedent (sibling modules of the crate root, `pub(crate)` for
      anything a parent or sibling needs — same rule confirmed working in
      A-1: private items defined in a module are already visible to all
      of that module's descendants, so e.g. `Template` and `load` stayed
      plain private and `new.rs`/`report.rs` see them for free). New
      files: `watch.rs` (`Watcher`, `fingerprint`, `watch_loop`, 274
      lines), `report.rs` (`parse_report`, `strip_position`,
      `diagnostics_report`, `watch_report`, `validate_file`, 218 lines),
      `new.rs` (`new_deck`, `prompt_line`, `interactive_new`,
      `starter_deck`, 150 lines), `templates.rs` (the three
      `serde_json::json!` starter-deck builders, 222 lines — kept as
      `json!` rather than `include_str!` assets per the plan, since they
      interpolate the author's deck name and `json!` handles the
      escaping). One re-export needed: `resume.rs` already did `use
      crate::fingerprint;` (crate-root path) before this split;
      `fingerprint`'s implementation moved into `watch.rs`, so `main.rs`
      re-exports it (`use watch::fingerprint;`) to keep `resume.rs`
      untouched — same technique as A-1's `content::indicator`
      re-export. The 16 `main.rs` tests split along exactly the lines
      they were already testing: 2 stayed in `main.rs` (they test
      `DEMO_DECK`), 3 moved to `new.rs` (they test `starter_deck`), 6 to
      `report.rs` (they test `parse_report`/`watch_report`), 5 to
      `watch.rs` (they test `Watcher`) — no test needed to move to
      `templates.rs`, since `starter_deck`'s tests already exercise the
      templates indirectly. `SPOTLESS_DECK` duplicated (a two-line JSON
      literal) rather than shared, since both test modules needing it are
      small and a shared test-fixture module would be more machinery than
      the duplication it removes. Verified: `cargo check -p fireside-cli
      --all-targets` was clean on the first attempt (no back-and-forth
      needed once the private-item-visible-to-descendants rule was
      applied consistently), `cargo clippy --workspace --all-targets -- -D
      warnings` clean, `cargo fmt --check` clean, `cargo test --workspace`
      195/195 (same total as before), full `scripts/verify.sh` green, a
      functional smoke test in a scratch dir (`fireside new --template
      linear`, `fireside validate`, `fireside validate --watch`, the
      no-args teaching text — the exact four dispatch paths whose
      call-targets moved) all behaved identically, and a tmux launch of
      `fireside demo` rendered and quit cleanly.
- [x] A-3 — incremental `insta` adoption — done 2026-07-17 (uncommitted
      on main). `insta = "1"` (v1.48.0 resolved) added to
      `[workspace.dependencies]`, consumed as a dev-dependency of
      `fireside-tui` only (`insta = { workspace = true }` — empty
      `[dev-dependencies]` table left over from the removed
      `pretty_assertions` now has a real entry); MSRV-1.88-verified via
      `cargo +1.88 check -p fireside-tui --all-targets` before touching
      any test code. Converted 5 of the 51 scenario tests per the plan's
      migration rule (whole-layout comparisons only, semantic `contains()`
      behavior contracts left untouched): the two named in the plan
      (`every_scene_renders_at_60x18` — 6 snapshots across one walk,
      `the_ending_is_centered_not_left_aligned`) plus three more that fit
      the same "card/reveal layout" description —
      `default_view_frames_the_slide_in_a_rounded_card`,
      `fullscreen_uses_the_full_width_not_the_measure`,
      `hidden_column_reserves_no_width_until_revealed_at_80x24` (2
      snapshots, before/after reveal). Explicitly left as-is:
      `the_card_is_the_same_stage_on_every_slide` and
      `ascii_art_code_block_centers_within_the_card_at_80x24` compare two
      renders against each other rather than pinning one golden layout,
      so they don't fit `assert_snapshot!`;
      `wide_terminals_keep_a_readable_measure` asserts a single precise
      x-coordinate via `buffer()`/`locate()`, already tight and
      mechanically different from the `screen()`-string tests. 11
      `.snap` files generated via `INSTA_UPDATE=always` (no `cargo-insta`
      binary needed) and eyeballed for sanity — e.g. the
      before/after-reveal pair visibly shows the second column appear.
      Proved the regression-catching claim the same way the fixture
      corpus was proved earlier in this project: hand-corrupted one
      `.snap` file's card corner, confirmed both plain `cargo test` and
      `cargo nextest run` (installed locally for this check — CI's actual
      runner, not just `cargo test`) fail with a readable diff and a
      `.snap.new` pending file, then restored it and reconfirmed green.
      Verified: 195/195 tests (`cargo nextest run --workspace`), clippy
      `--all-targets -D warnings` clean, fmt clean, full
      `scripts/verify.sh` green.
- [x] A-4 — proptests — done 2026-07-18 (uncommitted on main). **This
      closes out Stream A end to end.** Discovered the core serde
      round-trip property the plan asked for
      (`graph_round_trips_through_json`, `fireside-core`) already existed
      from spec 008 — its generator already covers all 7 `ContentBlock`
      kinds, nested containers via `prop_recursive`, and `reveal` on
      every variant — so no new work was needed there; verified rather
      than re-added. Added the three genuinely-missing properties: (1)
      `reveal_levels_are_sorted_deduped_and_positive`
      (`fireside-core::model::tests`) — bumped the existing
      `arbitrary_node` generator to `pub(super)` and reused it directly.
      (2) `reveal_state_stays_valid_and_next_back_are_consistent`
      (`fireside-engine::session::tests`) — added a new
      `arbitrary_reveal_graph_and_ops` generator (layers reveal-bearing
      content onto the existing navigation generator via `prop_map`,
      deliberately kept separate from `arbitrary_graph_and_ops` per that
      generator's own doc comment, which says content is empty on
      purpose for the history property) checking, per arbitrary op: reveal
      state is always `0` or a real level the current node declares;
      `next()` while a reveal is pending always reveals and never moves
      the node (FR-007); and `back()` always undoes a moving `next()`
      back to the same node id (though not its reveal progress, which
      resets by design on any node entry). (3)
      `validate_never_panics_and_only_names_real_nodes`
      (`fireside-engine::validation::tests`) — a new, deliberately
      separate generator (crate-boundary and `#[cfg(test)]`-privacy rule
      out reusing either of the other two crates' generators, per the
      precedent already documented in `session.rs`'s own generator
      comment) biased toward the shapes the validator specifically
      checks: a 3-id alphabet so duplicates/dangling targets are common,
      occasional `next`+`branch-point` conflicts, and container nesting
      bounded to 10 (just past the depth-8 limit) so both at-limit and
      over-limit shapes actually occur. Proved all three new properties
      catch real regressions, not just that they pass — the same
      discipline as A-3's snapshot check and the original spec 008
      property tests: hand-injected one bug per property (disabled the
      `next()` reveal gate; removed `reveal_levels`'s sort/dedup; made
      `container_depth` panic past a shallow threshold), confirmed each
      failed with a small, readable shrunk counterexample, then reverted.
      198/198 tests (up from 195), clippy `--all-targets -D warnings`
      clean, fmt clean, full `scripts/verify.sh` green.
- [x] B-2 — protocol parity in CI — done 2026-07-18 (uncommitted on main).
      Added two steps to the `validate` job in `models.yml`, after the
      existing `tsp-output` zero-diff check: `node run-fixtures.mjs` (Rust
      side already covered separately by engine fixture tests in
      `rust.yml`) and `node validate.mjs ../docs/examples/hello.json`
      (relative path since the job's `working-directory` default is
      `protocol`). Widened the job's `paths` filter to include
      `docs/examples/**` so an example-only edit still triggers the check.
      Verified both commands locally first: 17/17 fixtures match, and
      `hello.json` validates with 0 errors/0 warnings (1 info, the known
      `dead-end-branch` note).
- [x] B-3 — Dependabot — done 2026-07-18 (uncommitted on main). New
      `.github/dependabot.yml`, four `package-ecosystem` entries (`cargo`
      at `/`, `github-actions` at `/`, `npm` at `/docs`, `npm` at
      `/protocol`), all weekly with a `minor-and-patch` update-type group
      per the plan (protocol npm bumps stay guarded by the existing
      tsp-output zero-diff gate in B-2/models.yml).
- [x] B-4 — docs build-once — done 2026-07-18 (uncommitted on main). The
      `validate` job now uploads the Pages artifact itself (guarded by
      `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`,
      right after its existing `npm run build`); `deploy` dropped its own
      checkout/setup-node/install/build steps entirely and just runs
      `deploy-pages@v4` against the artifact `needs: validate` already
      produced. Halves docs CI work on every main push.
- [x] B-5 — coverage (informational) — done 2026-07-18 (uncommitted on
      main). New `coverage` job in `rust.yml`: `llvm-tools-preview` +
      `cargo-llvm-cov`/`nextest` via `taiki-e/install-action`, then three
      steps over one instrumented run rather than `--lcov` and
      `--summary-only` on a single invocation (the two are different
      report-output modes and don't compose): `cargo llvm-cov nextest
      --workspace --no-report` to collect profile data once, `cargo
      llvm-cov report --summary-only >> $GITHUB_STEP_SUMMARY` for the
      human-readable table, `cargo llvm-cov report --lcov --output-path
      lcov.info` + `actions/upload-artifact@v4` for the lcov file.
      `continue-on-error: true` on the job — informational only, no
      baseline/gate yet per the plan. Deferred to a follow-up session per
      user decision 2026-07-18: B-6 (Claude review workflow — needs an
      `ANTHROPIC_API_KEY` repo secret) and B-7 (release.yml — needs a
      rehearsal tag). All four `.github/workflows/*.yml` + the new
      `dependabot.yml` validated with `yaml.safe_load` (not run in CI
      itself, just local syntax sanity).

## Context

The user requested a six-area research engagement: (1) architecture/Rust best practices, (2) full documentation review, (3) build/deploy evaluation, (4) AI capabilities for the dev workflow, (5) a personal Rust learning path (Node/TS-first background, C# secondary), (6) ASCII art support. Three parallel repo audits (architecture, docs, CI/build) plus external research were completed. The user has decided:

- **ASCII art**: additive protocol change (spec bump → 0.1.3, ADR, symmetric Rust+Node validator rules) — follows the reveal precedent (ADR-009).
- **Releases**: minimal — tag-triggered workflow, macOS+Linux binaries on GitHub Releases. No cargo-dist/Homebrew/crates.io/changelog automation.
- **First wave**: code-health refactors + CI hardening. Docs and ASCII art follow.
- **Learning path**: `notes/rust-learning-path.md`, gitignored, written primarily for a Node/JS/TS developer.

## Executive summary of findings

**Architecture (healthy — targeted fixes only).** Clean 4-crate workspace (core→engine→tui→cli), no real dependency cycles, constitution rules verifiably enforced: zero `unwrap`/`expect` in library code, textbook thiserror/anyhow stratification, TEA invariant intact, `//!` on every file. Gaps: `crates/fireside-cli/src/main.rs` (1,096 lines — CLI + watcher + diagnostics formatting + 3 near-duplicate ~60-line deck templates), `crates/fireside-tui/src/render/mod.rs` (1,888 lines, #1 graph hub, 51 scenario tests with hand-rolled `screen()` string comparisons), no `insta` snapshot testing, only 2 property tests, zero trait definitions (fine at this scale; noted, not actioned).

**Docs (bimodal).** The protocol spec site is **already Astro + Starlight** — the 2026 industry-standard docs stack — current through 2026-07-17 and worth keeping as-is. **No migration needed; the "alternatives to Astro" question is resolved: keep Starlight.** The real problem is absence, not rot: there is **no user guide for the presenter/CLI at all**; README omits `import`, all flags (`--watch`, `--restart`, `--template`, `--author`), install/MSRV, and has no badges; `docs/README.md` describes a site structure that doesn't exist; the demo GIF (2026-07-11) predates reveal/quick-edit/timer and there's no regeneration script. Verdict: **salvage + fill gaps, do not rewrite.**

**Build/CI (fast but incomplete).** Green run ≈ 1.5 min warm; macOS test job is the critical path. Gaps by severity: no release process at all (zero tags/binaries/changelog); no Dependabot; `.cargo/config.toml` contradicts `Cargo.toml` release profile (`strip=false` vs `true`) and sets an invalid `pipelined-compilation` key; protocol parity checks (`run-fixtures.mjs`, `validate.mjs`) run only in `scripts/verify.sh`, never CI; CI clippy lacks `--all-targets` (weaker than local); no coverage; docs site builds twice per deploy; no LLM review in CI.

**Top 3 strategic recommendations**
1. Close the CI/local gap and ship the minimal release pipeline — the project targets non-technical presenters who cannot `cargo run`.
2. Adopt `insta` before splitting the god files, then split — the 51-test scenario suite is the safety net for every future refactor.
3. Write the missing user guide + regenerate VHS demos — the tool under-claims itself badly; every shipped P0/P1 feature is invisible in prose docs.

---

## Stream ordering & dependencies

1. **Wave 1 = Streams A + B in parallel**, except B-1 (clippy `--all-targets` alignment) lands **before** A's refactors merge, so CI actually lints the moved test code.
2. **B-2 (protocol parity in CI) is a hard prerequisite for Stream C** — C changes the wire format, and validator drift is invisible until parity runs in CI.
3. **Wave 2 = Docs (D) + ASCII art (C)**; C starts with the MSRV spike and goes through `/speckit-specify` as spec `009-ascii-art`.

## Wave 1 — Stream A: code health refactors (pure refactor, zero behavior change)

**Order: A-1 render split → A-2 main.rs split → A-3 insta → A-4 proptests.** Split before insta because the 51 scenario tests never call draw helpers directly — every assertion goes through `screen()`/`buffer()` → public `draw()` — so re-exporting moved items from `mod.rs` leaves all 51 hand-verified assertions untouched as the regression oracle for the move. Insta first would rewrite the safety net right before the move and churn snapshot names (insta names by module path).

**A-1: split `render/mod.rs` (M).** Production helpers (`:32–837`) move to new siblings under `crates/fireside-tui/src/render/`: `header.rs` (draw_header, header_rail), `content.rs` (node_lines, content_inner, end_marker, draw_content, indicator, notes_panel, draw_notes), `footer.rs` (draw_footer + draw_timer — footer and fullscreen both call it), `overlays.rs` (draw_edit, edit_line, draw_help), `hits.rs` (rect_contains, branch_option_hit, map_row_hit — keep re-exported, they're pub). `mod.rs` (~200 lines) keeps `draw()`, `areas()`, `surface()`, `apply_hyperlinks()`, `max_scroll()`, `overlay_rect()`, `MEASURE`, plus `pub(super) use` re-exports so the test module's `use super::*` resolves unchanged. Tests move verbatim to `render/tests.rs` (`#[cfg(test)] mod tests;` — module path identical, snapshot-safe). Risk: dead-code warnings under `-D warnings`; prefer selective `pub(super) use` over globs. Verify: `scripts/verify.sh` + **tmux smoke** (constitution requirement).

**A-2: split `main.rs` (M).** Follow the `import.rs`/`resume.rs` precedent: `watch.rs` (Watcher, fingerprint, watch_loop), `report.rs` (parse_report, strip_position, diagnostics_report, watch_report, validate_file — pure string formatting, now unit-testable), `new.rs` (new_deck, prompt_line, interactive_new, starter_deck), `templates.rs` (the three template fns). `main.rs` shrinks to ~250 lines of clap types + dispatch. **Keep templates as `serde_json::json!` functions, not `include_str!` assets** — they interpolate the author `name` into several JSON strings; `json!` handles escaping, an asset with placeholder substitution wouldn't. The duplication is content (three genuinely different starter decks), not code. Existing codebase principle holds: parameterless decks are assets (`assets/demo.fireside.json`), parameterized are `json!`. Add a test asserting each `Template` variant parses to a `Graph` with zero diagnostics. Split `mod tests` (main.rs:762) along the same lines.

**A-3: incremental insta adoption (S).** Add `insta = "1"` to workspace deps, dev-dep of `fireside-tui` only (confirm MSRV 1.88 in the same PR). `screen()` already returns a full-frame String — `insta::assert_snapshot!` is a drop-in. **Migration rule: convert only whole-layout comparisons** (e.g. `every_scene_renders_at_60x18`, `the_ending_is_centered_not_left_aligned`, the card/reveal layout tests); leave semantic `contains()` assertions as-is — they're behavior contracts and snapshotting would weaken them. Snapshots in `render/snapshots/` (insta default); nextest fails on stale snapshots, no CI change needed. New layout tests use insta going forward; convert stragglers opportunistically.

**A-4: proptests (S–M).** Extend existing generators (engine already has `proptest_support::arbitrary_graph_and_ops()` at `session.rs:251` and one property block). Add: core serde round-trip (arbitrary Graph → to_json → from_json → equal; generator must cover all 7 kinds incl. nested containers and reveal) and `reveal_levels()` ascending/deduped/no-zero; engine `back()` inverts `next()`, `reveal_step ≤ reveal_levels().len()` under arbitrary ops, `next()` on unrevealed node doesn't move; validation `validate()` never panics and diagnostics reference real node ids.

## Wave 1 — Stream B: CI hardening + minimal release

**B-1: correctness fixes (S), first.** (1) `rust.yml:48` clippy → `--workspace --all-targets -- -D warnings`; MSRV job → `cargo check --workspace --all-targets` (match verify.sh). (2) `.cargo/config.toml`: delete the whole `[profile.release]` block and the invalid `pipelined-compilation` key — **Cargo.toml's `strip = true` wins** (profiles belong in the manifest; the config override was silently shadowing it). Keep only aarch64 rustflags + dev/test opt-levels. (3) Remove the `libfontconfig1-dev` apt installs in all three rust.yml jobs — nothing in Cargo.lock uses fontconfig; confirm green CI on the PR. (4) Bump `checkout@v4`→`@v5`, `setup-node@v4`→`@v5` everywhere.

**B-2: protocol parity in CI (S).** `run-fixtures.mjs` is pure Node (Rust side already covered by engine fixture tests in rust.yml). Add two steps to the existing `validate` job in `models.yml`: `node run-fixtures.mjs` and `node validate.mjs ../docs/examples/hello.json`; widen its `paths` filter with `docs/examples/**`.

**B-3: Dependabot (S).** `.github/dependabot.yml`, weekly, grouped minor+patch: cargo `/`, github-actions `/`, npm `/docs`, npm `/protocol` (TypeSpec bumps are guarded by the existing tsp-output zero-diff gate).

**B-4: docs build-once (S).** `validate` job uploads the Pages artifact on main pushes; `deploy` job drops its rebuild (docs.yml:67–81) and keeps only `deploy-pages@v4`. Halves main-branch docs CI.

**B-5: coverage (S–M).** New ubuntu `coverage` job in rust.yml: `cargo llvm-cov nextest --workspace --lcov` + summary to `$GITHUB_STEP_SUMMARY`, lcov as artifact. **Informational, not gating** — no baseline exists and Stream A is about to move thousands of lines; revisit gating after A-4.

**B-6: Claude review workflow (S).** `.github/workflows/claude-review.yml` with `anthropics/claude-code-action@v1` on `pull_request`, `permissions: contents: read, pull-requests: write`, prompt pointing at `.specify/memory/constitution.md` (unwrap ban, dep allowlist, token styling, protocol symmetry). Non-gating. **Requires adding `ANTHROPIC_API_KEY` repo secret — user action.**

**B-7: minimal `release.yml` (M).** Trigger `push: tags: ['v*']`, `permissions: contents: write`. Matrix: macos-latest builds `aarch64-apple-darwin` + `x86_64-apple-darwin` (rustup target add, cross-compile on arm runner); ubuntu builds `x86_64-unknown-linux-gnu` + `x86_64-unknown-linux-musl` (musl-tools; Cargo.lock shows no C deps — syntect uses fancy-regex via two-face — so static musl should be pure-Rust; drop it if the first run disproves). Per target: `cargo build --release --locked -p fireside-cli --target $T`, package `fireside-<tag>-<target>.tar.gz` (binary + LICENSE + README). Release job: download artifacts, `sha256sum *.tar.gz > SHA256SUMS`, `gh release create "$GITHUB_REF_NAME" --generate-notes *.tar.gz SHA256SUMS`. Risks: `-ld_prime` rustflag in `.cargo/config.toml` is deprecated on newer Xcode — check first build's linker output, likely remove; **rehearse with a throwaway tag (e.g. `v0.0.1-rc1`, deleted after) before the real one**.

## Wave 2 — Documentation (Stream D)

All on the existing Starlight site (`docs/`) + repo files. No stack change.

1. **Fix drift (S)**
   - `README.md`: add `import` subcommand; document flags (`present --restart`, `validate --watch`, `new --template/--author`); add install section (GitHub Releases binaries once Stream B ships, plus `cargo install --path`), MSRV 1.88 note, CI/license badges; refresh feature list (quick-edit, reveal, timer, resume).
   - Rewrite `docs/README.md` to describe the *actual* site structure (it currently claims nonexistent `schemas/`, `decisions/`, three guides, autogenerated sidebar).
   - `docs/astro.config.mjs`: fix the sidebar §4→§6 gap; verify/correct the `tiberius` owner in `site`/`editLink`/social URLs against the real GitHub remote.
2. **New user guide (M)** — new Starlight guide group "Using Fireside" (separate from the protocol-authoring guide): installation & terminal requirements (truecolor, min size, font); quickstart (`fireside demo`, `fireside <file>`); creating decks (`new` interactive + `--template` variants, `import` from Markdown incl. frontmatter/slide syntax and reveal markers); presenting (full keybinding table from `crates/fireside-tui/src/render/mod.rs:747` help overlay — include the undocumented `h`/`g` aliases or remove them for consistency); `validate` + `--watch`; quick-edit modal; timer; resume. Source of truth for keys/flags: the clap definitions in `crates/fireside-cli/src/main.rs` and `app.rs` handlers — cite and cross-check, don't paraphrase from memory.
3. **VHS demos (M)** — add reveal blocks to `crates/fireside-cli/assets/demo.fireside.json`; extend/split `.github/demo.tape` into feature tapes (present+reveal, quick-edit, import, validate --watch, timer/map); add `scripts/demos.sh` to regenerate all tapes (`cargo build --release && vhs …`); embed the GIFs in the README and the new user guide. Optional CI job to rebuild GIFs on tape changes (deferred — VHS in CI is slow; script is enough).
4. **Housekeeping (S)** — gitignore `docs/dist/`; add `CHANGELOG.md` (hand-written per release, matching the minimal release decision); surface ADRs: either copy `.claude/adrs/` into the site's sidebar as a "Decisions" group or link them from `docs/README.md`. Add doctests only opportunistically (crates are internal; rustdoc already strong — low priority).

## Wave 2 — ASCII art feature (Stream C)

Architectural pre-work below feeds `/speckit-specify` as spec `009-ascii-art`. Context: ASCII art today is a language-less code block that the TUI already centers/clips (spec 005). Per user decision: additive protocol 0.1.3 + ADR + symmetric validators.

**C-1: Protocol shape — one new `ascii-art` block kind; all generation at authoring time; reject a render-time `big-text` heading attribute.** Rationale: a `big-text` attribute rendered via tui-big-text would add a runtime dep to `fireside-tui` (the crate ADR-008 was protecting), make rendering engine-divergent, and couple us to ratatui version lockstep. Instead, figlet at authoring time covers the "big title" use case, so a pre-rendered `ascii-art` kind subsumes it with **zero new renderer dependencies** — the TUI reuses the existing spec-005 centered-monospace path in `render/blocks.rs`. Failure-isolated: if a conversion crate fails the MSRV spike, only a CLI convenience is descoped; protocol and renderer are untouched. Wire shape: TypeSpec `AsciiArtBlock` extending the `RevealableBlock` base — `kind: "ascii-art"`, `art: string` (pre-rendered multi-line text), `alt?: string`; add `v0_1_3` to `Versions`, update "7 block kinds" prose to 8. Rust: `ContentBlock::AsciiArt { reveal: Option<u32>, art: String, alt: Option<String> }` + match arms in `reveal()`/`children()`. **ADR must state the compat caveat: a new *kind* (unlike 0.1.2's additive field) means pre-0.1.3 engines fail to deserialize decks using it.**

**C-2: Validation rules (symmetric, per the `reveal-masked-by-container` precedent):** `ascii-art-too-wide` WARNING when widest line > 76 cols (document as "80-col terminal minus card chrome", not TUI-coupled); `ascii-art-empty` WARNING; optionally `ascii-art-too-tall` (~18 lines, can ship without). New fixtures in `protocol/fixtures/` + `fixtures.expected.json` — trustworthy only because B-2 put parity in CI.

**C-3: MSRV 1.88 spike gate (Task 0 → ADR-011).** ADR-008 methodology: throwaway scratch project, resolve against ratatui 0.30 where relevant, `cargo tree` + **real `cargo +1.88 build`** (metadata lied by omission last time). Crates: `figlet-rs` (text→banner; zero deps, likely GO but check maintenance — fallback: embed one FIGfont + minimal in-repo renderer, or descope), `artem` and `rascii_art` (image→ASCII; **highest risk**, both pull the `image` crate — fallback: descope image conversion entirely, users paste externally generated art; protocol shape unaffected), `tui-big-text` (record result for the ADR even though not needed under the recommended design). Protocol change gets its own ADR-012 (per ADR-009 precedent).

**C-4: Conversion lives in `fireside-cli` only** (`fireside art` subcommand and/or `new`/`import` hooks in the post-A-2 module layout, e.g. alongside `templates.rs`). Keeps `image`-family crates out of `fireside-tui` entirely. **Requires a constitution amendment**: Principle III allowlist row for fireside-cli gains `figlet-rs` (+ chosen image crate if GO), ADR-gated like the pulldown-cmark precedent (ADR-006). The fireside-tui row is unchanged — that's the design's headline property.

**C-5: Reveal — yes, uniformly.** `ascii-art` carries the standard optional `reveal` field; once the two match arms exist it participates in `reveal_levels()` and `reveal-masked-by-container` automatically. Atomic reveal (whole block); per-line reveal out of scope.

Files touched: `protocol/main.tsp`, regenerated `tsp-output/` (zero-diff gate), `protocol/validate.mjs`, fixtures, `fireside-core/src/model/mod.rs`, `fireside-engine/src/validation.rs`, `fireside-tui/src/render/blocks.rs`, `fireside-cli` (templates.rs + new art module), constitution allowlist, ADR-011/012. Effort: spike S, protocol+validators M, TUI arm S, CLI authoring M — total M–L.

## Ongoing — AI capabilities (Stream E)

Already in place and working: clippy/rustfmt/rust-analyzer, cargo-nextest, cargo-audit + cargo-deny (weekly cron), proptest, graphify knowledge graph, Spec Kit pipeline, tmux smoke-test discipline. Additions, by ROI:

1. **`anthropics/claude-code-action` PR review workflow (S)** — the repo already has heavy Claude tooling locally but nothing in CI. Add a review workflow on `pull_request` (needs `ANTHROPIC_API_KEY` secret). Start review-only (no auto-fix). Part of Stream B.
2. **Coverage via `cargo-llvm-cov` (S)** — informational first (upload to the job summary / Codecov), gate later if signal is good. Part of Stream B.
3. **Dependabot (S)** — cargo + github-actions + npm (docs/) ecosystems, grouped monthly. Part of Stream B.
4. **Property-based testing expansion** — proptest already a dev-dep; Stream A adds serde round-trip + traversal-invariant proptests. Consider `cargo-mutants` mutation testing later as an occasional audit, not CI (runtime cost).
5. **Deferred/rejected**: cargo-fuzz (needs nightly; validator input is JSON — proptest round-trips cover most of it), semantic-release for Rust (contradicts minimal-release decision), AI test generation in CI (Spec Kit + TDD skill already covers authoring-time).

## Continuous — Rust learning path (Stream F)

Deliverable: `notes/rust-learning-path.md` (+ add `notes/` to `.gitignore`). Written for a **Node/TS-first** developer; C# comparisons only where they're the better bridge (generics/LINQ↔iterators, `async`/`Task` ↔ `Future`). Structure to write:

- **Phase 1 — Mental models (wk 1–3):** ownership/borrowing/moves vs JS reference semantics (`let` bindings move, no GC; closures capture by move/borrow vs JS closures capture-by-reference); `Option`/`Result` vs `null`/`undefined`/exceptions and `?` vs `try/catch`; enums+`match` vs TS discriminated unions (the closest bridge — Fireside's `ContentBlock` kind-tagged serde enums map 1:1 to tagged unions); traits vs TS structural interfaces (nominal, explicit impl). Resources: The Rust Book ch. 1–10 (4–6, 8–10 are the core), Rustlings alongside each chapter, "Rust for JavaScript developers" comparisons. Exercises: read `crates/fireside-core/src/model/mod.rs` and map every type to the TS you'd write; do Rustlings `ownership`/`enums`/`error_handling`.
- **Phase 2 — Std lib & idioms (wk 4–6):** iterators vs array methods (`map`/`filter`/`collect` — lazy, zero-alloc); `String` vs `&str` (no JS analogue — biggest early friction); collections; modules/crates vs ESM packages; `serde` vs `JSON.parse` (typed, derive-based). Resources: Book ch. 7, 11, 13; Rust by Example; Exercism Rust track (do ~10 mediums). Exercises in-repo: write one new validation rule in `crates/fireside-engine/src/validation.rs` with tests (small, pattern-following); add a proptest.
- **Phase 3 — The hard parts (wk 7–10):** lifetimes (only what's needed — Fireside barely uses explicit ones); trait objects vs generics; error design (`thiserror`/`anyhow` — study Fireside's own stratification as the worked example); `async` deliberately **skipped** (Fireside is sync; Book ch. 17 later if needed). Resources: Book ch. 15, 17(smart pointers), 19; "Effective Rust" (free online); Jon Gjengset's "Crust of Rust" videos for lifetimes/iterators.
- **Phase 4 — Fireside as the lab (ongoing):** implement Wave 1 refactor tasks yourself with me reviewing (template dedupe is the ideal first solo task — pure data manipulation, no lifetimes); then a Stream C task; then a ratatui widget change. Pitfall list for JS devs: fighting the borrow checker with `clone()` everywhere (fine at first, revisit later), expecting inheritance, `unwrap()` reflex (banned in this repo's library code — good forcing function), stringly-typed thinking.

## Verification

- After each Stream A refactor: `scripts/verify.sh` green (it is stricter than CI), plus a tmux real-terminal smoke of `fireside demo` per the constitution's test discipline (TestBackend can't catch timing bugs — established project learning).
- Stream B: push a `v0.1.0-rc` tag on a branch/fork to exercise release.yml end-to-end before the real tag; verify a downloaded binary runs `fireside demo` on macOS.
- Stream C: `/speckit-analyze` after tasks; fixture parity (`node protocol/run-fixtures.mjs`) green in the new CI job.
- Docs: `npm run check && npm run build` in `docs/`; click through deployed Pages site; run `scripts/demos.sh` and view GIFs.
- Learning path: reviewed by the user; first exercise (template dedupe) actually completed against it.

## Appendix — key external references

- ASCII crates: [tui-big-text](https://crates.io/crates/tui-big-text) (ratatui-org, font8x8), [figlet-rs](https://crates.io/crates/figlet-rs), [artem](https://docs.rs/artem), [rascii_art](https://github.com/orhnk/RASCII) — MSRV/compat spike gates in Stream C.
- Docs stack: keep [Starlight](https://docsio.co/blog/starlight-docs) (already in use; 2026 default recommendation over Docusaurus/mdBook for non-React teams).
- AI in CI: [anthropics/claude-code-action](https://github.com/anthropics/claude-code-action).
- Learning: The Rust Book, Rustlings, Rust by Example, Exercism Rust track, Effective Rust, Crust of Rust.
