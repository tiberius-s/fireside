# TASK013 — .github/ Agentic Tooling Overhaul

**Status:** Completed
**Added:** 2026-02-20
**Updated:** 2026-02-20

## Original Request

Review all `.github/` directory content related to agentic workflows and tooling.
Update copilot instructions to improve AI performance. Leverage calling other models as
subagents when better qualified. Improve existing skills and create new skill ideas.
Houseclean the memory-bank using the PRD method to organize things more accurately.

## Thought Process

The project had reached a fully-completed state (all 6 improvement phases done) but the
`.github/` infrastructure and memory-bank documents were stale — reflecting in-progress
work that was long since finished. This created a significant risk: any new AI session
would read the memory bank and misunderstand the project's actual state.

Additionally, the copilot instructions had several inaccuracies (wrong docs directory,
missing AppMode transitions, duplicate sections, no skills registry), and there were no
specialized agents for Rust expertise or structured skills for ADR writing or full
protocol cascade changes.

The strategy was: (1) clean up all documentation to reflect current reality, (2) add
missing infrastructure (skills, agents) proactively, (3) fix copilot instructions so
future AI sessions start with accurate context.

## Implementation Plan

- [x] Rewrite `memory-bank/projectbrief.md` with full PRD structure
- [x] Rewrite `memory-bank/activeContext.md` as current-state snapshot
- [x] Condense `memory-bank/progress.md` to status tables
- [x] Update `memory-bank/techContext.md` with full current stack
- [x] Expand `memory-bank/systemPatterns.md` with all patterns
- [x] Fix `memory-bank/tasks/_index.md` — mark TASK001-006 completed
- [x] Update `.github/copilot-instructions.md` — structural fixes + new sections
- [x] Create `.github/skills/adr/SKILL.md` — ADR generation skill
- [x] Create `.github/skills/protocol-change/SKILL.md` — full cascade skill
- [x] Create `.github/agents/rust-expert.agent.md` — Rust expert subagent

## Progress Tracking

**Overall Status:** Completed — 100%

### Subtasks

| ID   | Description                    | Status   | Updated    | Notes                                                         |
| ---- | ------------------------------ | -------- | ---------- | ------------------------------------------------------------- |
| 7.1  | Rewrite projectbrief.md        | Complete | 2026-02-20 | PRD structure with KPIs, scope, domain model table            |
| 7.2  | Rewrite activeContext.md       | Complete | 2026-02-20 | Current-state snapshot replacing historical log               |
| 7.3  | Condense progress.md           | Complete | 2026-02-20 | Status tables by layer replacing changelog                    |
| 7.4  | Update techContext.md          | Complete | 2026-02-20 | Full stack table, CI table, build commands, dependency policy |
| 7.5  | Expand systemPatterns.md       | Complete | 2026-02-20 | TEA diagram, AppMode FSM, crate bounds, all patterns          |
| 7.6  | Fix \_index.md                 | Complete | 2026-02-20 | TASK001-006 moved to Completed                                |
| 7.7  | Update copilot-instructions.md | Complete | 2026-02-20 | 8 targeted fixes + skills registry + routing guidance         |
| 7.8  | Create ADR skill               | Complete | 2026-02-20 | Nygard-style format, filing, project constraints              |
| 7.9  | Create protocol-change skill   | Complete | 2026-02-20 | 5-phase TypeSpec→Rust→docs cascade with checklist             |
| 7.10 | Create rust-expert agent       | Complete | 2026-02-20 | MSRV/boundary enforcement, Context7-first verification        |

## Progress Log

### 2026-02-20

Completed all 10 subtasks in a single session. Key decisions:

- **Memory bank rewrites**: Used PRD structure for `projectbrief.md` to give future AI
  sessions a clear, structured understanding of the project's current state and goals.
  Rewrote `activeContext.md` from a historical changelog to a current-state snapshot —
  the old approach accumulated noise that buried relevant context.

- **copilot-instructions.md fixes**: Removed two duplicate sections (TypeSpec Workflow
  and When Making Changes appeared twice each). Fixed docs directory reference
  (`decisions/` → `crates/`). Added `GraphView` to AppMode FSM. Added nextest as
  primary test command with githooks install reference. Added Skills Registry and
  subagent routing guidance so future sessions know when to invoke each skill or agent.
  Removed the redundant `typespec-build` entry now that `protocol-change` covers
  the same ground.

- **ADR skill**: Nygard-style format with Fireside-specific constraint table. Filing
  location: `docs/src/content/docs/explanation/adr-NNN-*.md`. Includes a worked example.

- **Protocol-change skill**: Full 5-phase cascade (TypeSpec → JSON Schema compilation →
  Rust struct update → documentation → 5 verification gates). Includes a checklist and
  common pitfalls table. This skill operationalizes the "TypeSpec first" rule in the
  copilot instructions.

- **rust-expert agent**: Models the Context7 agent pattern. Adds MSRV validation,
  crate boundary enforcement, and TEA/index-rebuild invariant checks. Includes a
  research workflow, output format template, and handoff pattern.
