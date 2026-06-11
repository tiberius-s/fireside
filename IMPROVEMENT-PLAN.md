# Fireside Improvement Plan

Date: 2026-06-10
Method: spec/docs/schema audit, full test-suite runs, CLI exercising, cross-validation with `protocol/validate.mjs`, Context7 API verification, graphify knowledge graph (1,286 nodes / 91 communities), and an audit of all AI configuration in `.github/` and `.claude/`.

## 1. Current State Summary

The protocol side is **internally coherent**: `protocol/main.tsp`, the generated schemas in `protocol/tsp-output/schemas/`, and the spec chapters in `docs/src/content/docs/spec/` all agree on the 0.1.0 model (explicit edges, `view-mode`, 2 transitions, 7 block kinds, NodeId-based history). The Rust workspace (~17k lines; the TUI alone is 13.8k = 81%) still implements the **pre-rewrite slide-deck model**. Git history confirms the sequencing: hooks were disabled "during protocol rewrite" (365ef85) and the rewrite landed in docs/TypeSpec but not in `crates/`.

The split-brain, reproduced locally:

| Document | `node validate.mjs` (protocol) | `fireside validate` (Rust) |
|---|---|---|
| `docs/examples/hello.json` (canonical) | ✓ 0 errors | ✗ parse failure |
| `fireside new` scaffold output | ✗ 2 errors, 3 warnings | ✓ valid |

Test status: `cargo test --workspace` fails — 1 CLI e2e test (`validate_hello_exits_zero`) and 2 TUI golden tests, all from one root cause (no string-shorthand `traversal` support). **CI impact: `.github/workflows/rust.yml` runs `cargo nextest run --workspace`, so the Rust CI job is red right now** — the disabled local hooks only hid this locally.

The graphify graph independently surfaced the two largest drift pairs as `semantically_similar_to` bridges: `Layout enum (12 variants)` ↔ `ViewMode enum (default, fullscreen)` and `Transition enum (8 variants)` ↔ `Transition (none, fade)` — both linking crate READMEs to spec chapters.

## 2. Spec Gaps and Drift

### Rust implementation vs. spec

| # | Finding | Type | Evidence | Fix |
|---|---|---|---|---|
| D1 | `traversal: NodeId \| Traversal` string shorthand unsupported — canonical example unparseable; breaks 3 tests | impl bug (highest priority) | `protocol/main.tsp:317` vs `crates/fireside-core/src/model/node.rs:54` | Custom string-or-object `Deserialize` (same visitor pattern as `ListItem`, `content.rs:113`) |
| D2 | Engine falls back to sequential advance on `next()`; spec: "no implicit sequential fallback" | impl bug (semantic) | `crates/fireside-engine/src/traversal.rs:99-107` vs `main.tsp:282-284`, traversal.md §Next step 5 | Remove fallback; terminal node = no-op |
| D3 | `next()` not blocked at branch points; spec: "next() is BLOCKED" | impl bug | `main.tsp:397`; engine `next()` never checks `branch_point()` | Gate `next()` when a branch point exists |
| D4 | `back()` walks sequentially backward when history is empty; spec: no-op | impl bug | `traversal.rs:115-119` vs traversal.md §Back step 1 | No-op on empty history |
| D5 | History stores array indices; spec invariant 5: node IDs | impl bug | `TraversalEngine.history: VecDeque<usize>`; `clamp_to_graph` silently corrupts history after edits | Store `NodeId`s |
| D6 | `Node.id` optional in Rust; spec: required | impl drift | `node.rs:22` vs `Node.json` required list | Make required; fix scaffold |
| D7 | `Traversal.after` field + precedence logic exists only in Rust | unspecified feature | `traversal.rs` core+engine; spec covers rejoin via explicit `next` (traversal.md §Branch return wiring) | Remove `after` (record as ADR) |
| D8 | Required Checks #3 (non-empty `options`) and #4 (`next`+`branch-point` mutually exclusive) never validated | impl bug | validation.md §4 vs `crates/fireside-engine/src/validation.rs` | Add both as Error severity |
| D9 | No `view-mode` anywhere in Rust (0 grep hits); obsolete 12-variant `Layout` enum used in 10+ TUI files | impl drift (biggest refactor) | `layout.rs` claims "Matches the 12 protocol layout modes" — false for 0.1.0 | Add `ViewMode`; migrate node layout semantics to container layouts (ADR) |
| D10 | `Transition`: 8 Rust variants vs 2 in spec | impl drift | `transition.rs` vs `main.tsp:78-84` | Parse unknown → `none` (spec-sanctioned fallback); emit only spec values |
| D11 | `BranchOption.key`: required `char` vs optional `string`; `description` missing | impl drift | `branch.rs:29` vs `BranchOption.json` | Optional `String` + add `description` |
| D12 | `extension` block, `ExtensionDeclaration`, graph `theme`/`font`/`tags` exist only in Rust | unspecified feature | `content.rs:87-98`, `graph.rs` vs `main.tsp` union of exactly 7 | Keep code, document as non-normative engine extras in a spec appendix; exclude from conformance (ADR) |
| D13 | `ImageBlock` missing `width`/`height`; `alt` is `String` not `Option`; `ListBlock` items nested objects vs spec `string[]` | impl drift | `content.rs` vs `main.tsp:147-177` | Add width/height; accept superset for lists, serialize strings when flat |
| D14 | `save_graph` discards original `defaults` and defaults are baked into nodes at load — edit/save round-trip mutates documents | impl bug (editor data integrity) | `crates/fireside-engine/src/loader.rs` `graph_to_file`: `defaults: Some(NodeDefaults::default())` | Preserve raw defaults; resolve the cascade at render time |

### Gaps in the spec itself

- **S1** — `ContainerBlock.layout` is a bare `string` in TypeSpec (`main.tsp:204`) though docs enumerate exactly `stack|columns|center`. Make it an enum so schema validation enforces it.
- **S2** — `ImageBlock.width` is `int32` documented as "columns or percentage" — ambiguous in an integer field. Specify units.
- **S3** — `validate.mjs` reports missing IDs as `Duplicate node ID "undefined"` instead of a Layer-1 "missing required `id`" schema error. The JS validator under-reports the schema layer.
- **S4** — The official validator warns `dead-end-branch` on the spec's own canonical example (`thanks` is a legitimate terminal node). Tune the lint or annotate the example.

## 3. Rust Implementation Progress by Crate

- **fireside-core — well-built, wrong model.** Clean serde modeling and roundtrip tests, but encodes the pre-rewrite protocol (D1, D6, D7, D9–D13). ~80% of conformance work lands here, and it is the smallest crate (~1k lines).
- **fireside-engine — solid mechanics, wrong semantics.** Good separation (loader/traversal/validation/session + editor command/undo system). Traversal works but implements slide-deck semantics (D2–D5). Validation misses 2 of the spec's 4 Required Checks (D8).
- **fireside-cli — good surface, untrustworthy validate.** Nice command set (`present/open/edit/new/validate/fonts/import-theme`). `validate` disagrees with the protocol validator in both directions; `new` scaffolds non-conforming documents; `present --plain` dumps raw JSON.
- **fireside-tui — large, polished, over-scoped.** 13.8k lines: themes, font management, theme import, 8 transition animations, graph view, full node editor, design tokens, templates. Tests healthy (89 unit + harness) except 2 golden tests blocked on D1. Most rendering targets the obsolete `Layout` model.

Build health: MSRV 1.88 / edition 2024, current deps (ratatui 0.30, serde 1, thiserror 2). Manifest warts: invalid `[workspace.dev-dependencies]` key (cargo warns; `pretty_assertions` is not actually shared) and an unused `build.pipelined-compilation` key in `.cargo/config.toml`.

Note on the graph report: the "1-file import cycles" listed in `GRAPH_REPORT.md` are AST self-loop artifacts (file → itself), not real architectural cycles. Don't chase them.

## 4. CLI/TUI UX Assessment

The interactive TUI was not launched in this assessment environment (no TTY); findings are from tests, `--plain` output, and source inspection.

- **Trust-breaking (worst):** `fireside validate` and the protocol validator disagree in both directions. An author following the docs gets a parse error on the canonical example; an author using `fireside new` gets a file the ecosystem rejects.
- **First-run path broken:** scaffold → validate → present works only inside the Rust bubble. Scaffold emits a `$schema` URL (`fireside.dev/...`) that doesn't resolve to the real schemas.
- **Diagnostics:** serde errors give line/column; validation.md asks for failing path + rule. The Rust validator emits neither schema paths nor lint codes (validate.mjs has codes like `[unique-node-ids]` — the CLI should match).
- **`present --plain`** prints raw JSON blocks — not a useful export. Low priority.
- **Keep:** the clap CLI organization, the editor command/undo system, and the TUI golden-test harness — the harness is the regression net for the Phase 4 refactor.

## 5. Context7 Verification (new since first draft)

Verified via Context7 against `jsonschema` 0.40 (docs.rs):

- Draft 2020-12 fully supported (`jsonschema::draft202012`, `validator_for`, `options()`).
- Reusable validators: build once, validate many — fits a CLI `validate` loop.
- Diagnostics: `validator.iter_errors(&instance)` yields errors with `error.instance_path()` — exactly the "failing path + rule" output validation.md requires.
- Local schema refs: `options().with_base_uri(...)` resolves `$ref` between the generated schema files (Graph.json → Node.json → ContentBlock.json), so the schemas can be embedded with `include_str!` and registered — no network.
- **Caveat:** default features pull in `reqwest` (HTTP resolving). Use `default-features = false, features = ["resolve-file"]` (or no resolve features at all with fully embedded schemas) to keep the dependency tree small and respect the engine crate boundary.
- **Unverified:** the crate's MSRV was not surfaced by the docs query. Before adopting, run a 5-minute spike: `cargo +1.88 check` with the dependency added. If it fails MSRV, fall back to semantic-parity-only validation (no new dep), which the plan supports.

All other recommendations use already-pinned dependencies (serde visitor patterns, ratatui 0.30) and need no new verification.

## 6. Phased Roadmap (3–6 weeks, small PRs)

### Phase 0 — Stop the bleeding (1–2 days)

1. Land D1 (traversal string shorthand) immediately — it alone turns CI green (all 3 failing tests share this root cause) and makes the canonical example loadable.
2. Fix `[workspace.dev-dependencies]` → per-crate `[dev-dependencies]`; remove the stale `.cargo/config.toml` key.
3. Add a conformance smoke test: Rust must parse `docs/examples/hello.json`.

### Phase 1 — Core model conformance (week 1)

One PR per drift item, all in `fireside-core`:

1. D6 required `id`; D11 BranchOption key/description; D7 remove `after`; D13 image width/height + list serialization.
2. D9 add `ViewMode`; keep `Layout` temporarily behind a load-time translation for the TUI.
3. D10 transitions: parse unknown → `none` fallback.

Acceptance per PR: `cargo test -p fireside-core && cargo test -p fireside-engine`.

### Phase 2 — Engine semantics (week 2)

1. D2/D3/D4: explicit-edge `next()` with branch gating; no-op terminal/back.
2. D5: NodeId-based history.
3. D8: the two missing Required Checks + recommended warnings (unreachable nodes, duplicate branch keys) with lint codes matching `validate.mjs`.

### Phase 3 — CLI trust (week 3)

1. Rewrite scaffold templates to conform (explicit ids, explicit traversal edges, string list items, real or no `$schema`).
2. D14 round-trip fidelity in `save_graph`.
3. `validate` parity: same codes/severities as `validate.mjs`. Optionally add the `jsonschema` crate for true Layer-1 validation (per §5 — spike MSRV first).
4. Cross-validator CI check: both validators must agree on shared fixtures (see §7, A5).

Acceptance: the table in §1 shows ✓/✓ and ✗/✗.

### Phase 4 — TUI realignment (weeks 4–5)

1. Map rendering to `view-mode` + container layouts; translate old `Layout` values on load with deprecation warnings.
2. Restore the 2 golden tests; add goldens for branch gating and terminal nodes.
3. Decide extension/theme/font fate (D12) via ADR: recommend documenting as non-normative engine features rather than deleting working code.

### Phase 5 — Spec polish + guardrails (week 6)

1. S1–S4 spec/tooling fixes; regenerate schemas (models.yml already verifies `tsp-output/` is committed).
2. Shared conformance fixture directory consumed by both `validate.mjs` and Rust tests.
3. Re-enable local git hooks (reverting 365ef85/c1c5df1 behavior) once the workspace is green.

Defer: new transitions/layouts, `present --plain` improvements, theme import polish — none block conformance.

## 7. AI & Agentic Workflow Audit (`.github/` + `.claude/`)

### What's working well

- **rust-expert.agent.md** is the strongest asset: correct Context7 tool names, a concrete MSRV/crate-boundary/idiom rulebook, a verification-before-advising workflow, and a checklist. The boundary table matches the actual workspace.
- **documentation-writer agent + prompt** are well-scoped (Diátaxis, accuracy-first, anti-padding).
- **The graphify PreToolUse hooks** in `.claude/settings.json` work as designed (observed firing during this audit) and nudge toward graph queries instead of raw grep.
- **CI structure** is sound: lint (fmt/clippy `-D warnings`/doc), test matrix with nextest, an MSRV 1.88 job, TypeSpec compile + committed-output verification, security audit + cargo-deny.

### Findings and recommendations

| # | Finding | Evidence | Recommendation |
|---|---|---|---|
| A1 | **Two parallel agent ecosystems with duplicated, divergence-prone rules.** The engineering constraints (MSRV, crate boundaries, TEA invariant, no-unwrap, error stratification) live in `.github/agents/rust-expert.agent.md` AND `.github/instructions/rust-best-practices.instructions.md` — but **Claude Code loads neither**; `CLAUDE.md` contains only graphify rules. Claude Code sessions operate without the project's core engineering constraints. | `CLAUDE.md`, `.claude/CLAUDE.md` vs `.github/` | Create one canonical constraints doc (e.g., `AGENTS.md` or a section in root `CLAUDE.md`) holding MSRV, boundary table, idioms, and error stratification. Have the Copilot agent/instructions files reference it instead of restating it. This is the highest-leverage AI-workflow fix. |
| A2 | **context7.agent.md is stale and off-target.** It mandates `mcp_context7_get-library-docs` — the current tool is `query-docs` (rust-expert uses the right name). Its examples are Express/React/Tailwind in a Rust/TypeSpec repo, and the "ALWAYS inform about upgrades" rule adds noise. | `.github/agents/context7.agent.md` | Fix the tool name; either trim it to a thin generic helper or fold its mandate into rust-expert and delete it. |
| A3 | **markdown.instructions.md contradicts itself and the repo.** Rule 7 says 400-char lines; the Formatting section says break at 80. It bans H1 headings while repo README/plan docs use them. It contains a `csharp` example. Meanwhile `.markdownlint.json` exists but no CI step runs markdownlint. | `.github/instructions/markdown.instructions.md`, `.markdownlint.json`, `docs.yml` | Make `.markdownlint.json` the single source of truth; rewrite the instruction file to defer to it; optionally add a markdownlint step to docs.yml. |
| A4 | **The planning prompt moved to a dead location.** `claude-fable-plan.prompt.md` was moved to `.claude/prompts/`, which neither tool reads: Copilot reads `.github/prompts/`, Claude Code reads `.claude/commands/` (slash commands) or skills. Its frontmatter (`tools: ['edit/editFiles', ...]`) is Copilot-format. | git status rename; `.claude/prompts/` | Either move it back to `.github/prompts/` (Copilot) or convert it to `.claude/commands/audit-roadmap.md` with Claude-style frontmatter so `/audit-roadmap` works. |
| A5 | **No cross-validator conformance job in CI.** models.yml compiles TypeSpec; rust.yml tests Rust — but nothing runs `protocol/validate.mjs` against `docs/examples/` or compares Rust validate output against it. The §1 split-brain shipped invisibly. | `.github/workflows/` | Add a `conformance` job: run `validate.mjs` on every file in `docs/examples/` and on `fireside new` output; once Phase 3 lands, also run `fireside validate` on the same fixtures and fail on disagreement. This is the guardrail that prevents drift from re-opening. |
| A6 | **Local hooks disabled and removed mid-rewrite, CI left red.** Commits 365ef85/c1c5df1 disabled hook test execution; the `githooks/` directory is now absent; rust.yml still runs the full suite and fails. | git log; `ls githooks/` empty | Phase 0 makes CI green; Phase 5 restores hooks. Consider lighter hooks (fmt + clippy only) since CI owns the test gate. |
| A7 | **Graph freshness depends on model memory.** Root CLAUDE.md says "After modifying code, run `graphify update .`" — an instruction the model can forget. The graphify skill ships a post-commit hook reference that is not installed. | `CLAUDE.md`; `.claude/skills/graphify/references/hooks.md` | Automate: add a PostToolUse hook on Edit/Write (or the post-commit hook) running `graphify update .` — it is AST-only, no API cost. Hooks are reliable; memory is not. |
| A8 | **Session permissions accumulating in settings.local.json.** Useful read-only commands (`cargo test *`, `cargo run *`, `node validate.mjs ...`, `graphify export *`) are in local settings only. | `.claude/settings.local.json` | Promote the stable, low-risk allowlist entries into the checked-in `.claude/settings.json` so every contributor's Claude sessions get fewer prompts. |
| A9 | **Roadmap decisions should flow through the ADR skill.** D7 (remove `after`), D9 (Layout → ViewMode), D12 (extension/theme/font as non-normative) are exactly the protocol-affecting decisions the `adr` skill exists for. | `.github/skills/adr/SKILL.md` | Open one ADR per decision before the implementing PR; link the ADR in the PR description. Cheap, and it gives smaller models an authoritative decision record to execute against. |

## 8. Token-Efficient Strategy for a Smaller Model

- **One drift ID per PR.** Each D-item is independently checkable with one command — hand a smaller model the table row, file path, and acceptance command; no exploration needed.
- **`hello.json` is the oracle.** Every Phase 1–3 task ends with `cargo test --workspace && node protocol/validate.mjs docs/examples/hello.json`. Pass/fail is unambiguous.
- **Patterns inline, not researched.** D1's fix is the visitor pattern already at `content.rs:113-159` — point at it. Serde kebab-case attributes are already established in the codebase.
- **Use graphify before grep.** `graphify query` returns scoped subgraphs that are far cheaper than reading the 13.8k-line TUI. The PreToolUse hooks already nudge this.
- **Sequence strictly: core → engine → CLI → TUI.** TUI work before the model stabilizes burns the most tokens (81% of the code, depends on everything else).
- **ADRs as execution contracts (A9).** A smaller model executing Phase 4 reads the Layout→ViewMode ADR instead of re-deriving the decision.

## 9. Execution Prompts

The roadmap is decomposed into 24 PR-sized, dependency-ordered task prompts in [`plan/tasks/`](plan/README.md). Each prompt is self-contained: goal, evidence, exact files, drift guardrails ("Do NOT" sections), and acceptance commands. Execute them in numeric order — the numbering *is* the dependency graph (tasks 19, 20, 22–24 are parallel-safe).

| Range | Track |
|---|---|
| 01–02 | Phase 0: manifest hygiene, D1 fix (turns CI green) |
| 03–08 | Phase 1: core model conformance (D6, D7, D9–D11, D13) |
| 09–11 | Phase 2: engine semantics + validation (D2–D5, D8) |
| 12–16 | Phase 3: scaffold, round-trip (D14), validate parity, optional jsonschema spike, conformance CI |
| 17–18 | Phase 4: TUI ViewMode/container rendering + traversal UX, golden tests |
| 19–21 | ADRs, spec fixes S1–S4, shared fixtures + hook restoration |
| 22–24 | AI workflow fixes (A1–A4, A7–A8) |

### Terminal UX design principles (binding for tasks 17–18 and all future TUI work)

- All styling flows through `DesignTokens` (`crates/fireside-tui/src/design/tokens.rs`) — no hardcoded colors, padding, or borders in render functions.
- Layout uses ratatui 0.30 primitives (verified via Context7): `Layout::vertical`/`horizontal` with `Constraint::Fill`/`Length` and `.spacing(1)` for container `stack`/`columns`; `Rect::centered(..)` or `Flex::Center` for `center`; unknown layout hints degrade to `stack`.
- Every state communicates its valid actions: footer key hints per mode (consistent ordering and separators), flash messages for blocked actions (`next` at a branch point), an explicit end-of-path indicator on terminal nodes.
- Every new visual state gets a golden test in `crates/fireside-tui/tests/harness_golden.rs` — the golden harness is the regression net for the Phase 4 refactor.

## 10. Honest Caveats

- The interactive TUI was not launched (no TTY in the audit environment); editor UX claims derive from `app_tests/` and source.
- `jsonschema` crate API verified via Context7; its MSRV was not, and is gated on a spike (§5).
- The "spec is right, Rust is behind" framing is an inference from git history and validator behavior. If any Rust extras (`after`, nested list items, extension blocks) are *intended* protocol features, they belong in `main.tsp` first — confirm the Phase 1 ADR decisions before executing.
- The graphify semantic layer is one extraction pass (~200k tokens); INFERRED edges (e.g., the 16 around `load_graph()`) are model-reasoned and should be treated as leads, not facts.
