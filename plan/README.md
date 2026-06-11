# Fireside Execution Plan

Companion to [`IMPROVEMENT-PLAN.md`](../IMPROVEMENT-PLAN.md). Each file in `tasks/` is a self-contained prompt for one PR-sized unit of work. Execute them **in numeric order** — the sequence is the dependency order, designed so a model never has to reason about unfinished neighboring work and cannot reintroduce drift.

## Rules for executing a task

1. Do exactly what the task file says. Do not expand scope, refactor neighboring code, or "fix" things other tasks own.
2. Every task ends with acceptance commands. All must pass before the task is done.
3. If a task's premise looks wrong (file moved, API changed), stop and report — do not improvise.
4. After completing a task, run `graphify update .` to keep the knowledge graph current.
5. One task = one PR. Use the task title as the PR title.

## Design principles (apply to every TUI task)

- **All visual styling flows through `DesignTokens`** (`crates/fireside-tui/src/design/tokens.rs`). Never hardcode a `Color`, padding, or border style in a render function.
- **Layout uses ratatui 0.30 primitives** (verified via Context7):
  - vertical stacking → `Layout::vertical([Constraint::Length(..), ..])`
  - columns → `Layout::horizontal` with `Constraint::Fill(1)` per child and `.spacing(1)`
  - centering → `Rect::centered(Constraint, Constraint)` or `Layout` with `Flex::Center`
- **Every user-visible state needs feedback**: blocked actions flash a message, terminal nodes show an end indicator, the footer always shows the keys valid in the current mode.
- **Every new visual state gets a golden test** in `crates/fireside-tui/tests/harness_golden.rs`.
- **Consistency beats novelty**: reuse existing chrome/footer/flash patterns (`ui/chrome.rs`, `App::flash`) rather than inventing new ones.

## Sequence and dependencies

| Task | Title | Depends on | Phase |
|---|---|---|---|
| 01 | Workspace manifest hygiene | — | 0 |
| 02 | Traversal string shorthand (D1) + hello.json smoke test | — | 0 |
| 03 | BranchOption: optional string key + description (D11) | 02 | 1 |
| 04 | Content blocks: image size, optional alt, list serialization (D13) | 02 | 1 |
| 05 | Transition unknown-value fallback (D10) | 02 | 1 |
| 06 | Remove `traversal.after` (D7) | 02 | 1 |
| 07 | Introduce `ViewMode` (D9, core only) | 02 | 1 |
| 08 | Require `Node.id` (D6) | 07 | 1 |
| 09 | Engine: explicit-edge traversal semantics (D2/D3/D4) | 03, 06 | 2 |
| 10 | Engine: NodeId-based history (D5) | 09 | 2 |
| 11 | Validation: required checks + lint codes (D8) | 06, 09 | 2 |
| 12 | Scaffold conformance | 08, 11 | 3 |
| 13 | Round-trip fidelity in save_graph (D14) | 07, 08 | 3 |
| 14 | `fireside validate` output parity | 11 | 3 |
| 15 | Optional: JSON Schema Layer-1 validation spike | 14 | 3 |
| 16 | CI conformance job | 12, 14 | 3 |
| 17 | TUI: ViewMode + container layouts via ratatui | 07, 09, 10 | 4 |
| 18 | TUI: traversal UX polish + golden tests | 17 | 4 |
| 19 | ADRs for protocol decisions | — (best before 17) | 4 |
| 20 | Spec fixes S1–S4 + schema regeneration | — | 5 |
| 21 | Shared conformance fixtures + restore git hooks | 16, 20 | 5 |
| 22 | AI workflow: consolidate engineering constraints | — | any |
| 23 | AI workflow: fix stale agent/instruction files | — | any |
| 24 | AI workflow: automate graphify + permissions | — | any |

Tasks 19, 20, 22–24 have no code dependencies and can run in parallel with the main track. Everything else is strictly sequential as listed.

## Drift guards (read before every task)

- The **spec is the source of truth**: `protocol/main.tsp`, the generated schemas in `protocol/tsp-output/schemas/`, and `docs/src/content/docs/spec/`. When code and spec disagree, the code changes.
- `docs/examples/hello.json` is the canonical document. It must parse, validate, and present correctly after every task from 02 onward.
- Never add a field, enum variant, or behavior that is not in the spec unless the task explicitly says it is a documented non-normative engine extra.
- Never re-add: `traversal.after`, sequential-advance fallback, index-based history, optional node ids.
