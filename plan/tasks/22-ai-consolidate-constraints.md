# Task 22 — AI workflow: consolidate engineering constraints

**Depends on:** none (parallel-safe)
**Crates:** none (AI config only)
**Phase:** any — highest-leverage AI fix (finding A1)

## Goal

One canonical engineering-constraints document that every AI surface (Claude Code, Copilot agents, Copilot instructions) reads, ending the current three-way duplication — and ending the gap where **Claude Code loads none of it**.

## Background

MSRV/crate-boundary/TEA/no-unwrap/error-stratification rules are duplicated in `.github/agents/rust-expert.agent.md` and `.github/instructions/rust-best-practices.instructions.md`. Root `CLAUDE.md` contains only graphify rules, so Claude Code sessions work without the project's core constraints.

## Steps

1. Create `AGENTS.md` at repo root containing, verbatim from rust-expert.agent.md (it has the best versions): the MSRV rule (1.88, edition 2024), the crate-boundary table, the mandatory idioms (no unwrap/expect in libs, `#[must_use]`, doc comments, TEA invariant, `rebuild_index()` rule, kebab-case serde), and the error-stratification table. Add: "spec is source of truth: protocol/main.tsp + tsp-output schemas + docs/spec" and the build/test commands (`cargo test --workspace`, `node protocol/validate.mjs <file>`, `cd protocol && npm run build`).
2. Root `CLAUDE.md`: add one line near the top — "Engineering constraints: read `AGENTS.md` and follow it for all Rust/protocol work." Keep the graphify section as is.
3. `.github/agents/rust-expert.agent.md`: replace the duplicated rule sections with "The canonical rules live in `/AGENTS.md` — load and enforce them"; keep only the agent-specific parts (Context7 workflow, output format, handoff).
4. `.github/instructions/rust-best-practices.instructions.md`: same treatment — keep the planning/maintainability checklist (unique content), point to AGENTS.md for the rules it duplicated.

## Do NOT

- Change any rule's substance while moving it (pure consolidation; substantive changes need their own PR).
- Delete the agent/instruction files.

## Acceptance

- `grep -c "1.88" AGENTS.md` ≥ 1; the boundary table appears exactly once across `AGENTS.md` + the two .github files (`grep -rn "Permitted dependencies" AGENTS.md .github/ | wc -l` == 1).
- Root `CLAUDE.md` references AGENTS.md.
