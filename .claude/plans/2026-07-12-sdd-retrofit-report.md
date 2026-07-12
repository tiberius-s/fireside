# Spec-Driven Development Retrofit — Research Report & Plan

Date: 2026-07-12. Scope: adopt SDD (GitHub spec-kit paradigm) in Fireside,
overhaul `.github/` + `.claude/`, and structure the workflow so a medium-cost
model (Haiku) can execute reliably with guardrails, reserving Fable/Opus for
spec-writing and review.

---

## 1. What spec-kit and mattpocock/skills actually are

Two assumptions in the task brief turned out to be wrong; correcting them
changes the plan, so they come first.

### spec-kit is a markdown-artifact pipeline, not `.spec.yml`

There are no YAML specs and no spec "runner" in spec-kit. It is:

- **A Python bootstrapper** (`specify` CLI, installed via `uv`) that scaffolds
  templates and slash commands into a repo for 30+ coding agents (Claude Code
  included).
- **Five slash commands** forming a linear pipeline:
  `/speckit.constitution` → `/speckit.specify` → `/speckit.plan` →
  `/speckit.tasks` → `/speckit.implement`, plus optional `/speckit.clarify`,
  `/speckit.analyze`, `/speckit.checklist`.
- **Markdown artifacts** it generates:
  - `.specify/memory/constitution.md` — immutable project principles, checked
    as a gate during planning ("constitution check").
  - `specs/NNN-feature-name/spec.md` — the *what/why* (user stories,
    functional requirements, explicitly no tech stack).
  - `specs/NNN-feature-name/plan.md` (+ `research.md`, `data-model.md`,
    `contracts/`) — the *how*.
  - `specs/NNN-feature-name/tasks.md` — small, ordered, individually
    verifiable task units.
- **"Running a spec"** means `/speckit.implement` walking `tasks.md` task by
  task. Ambiguities are marked `[NEEDS CLARIFICATION]` in the spec and must be
  resolved before planning proceeds — that is the escalation mechanism.

The methodology doc (`spec-driven.md`) is explicitly greenfield-oriented. It
offers **no brownfield/retrofit guidance** — flagged as an unknown we resolve
ourselves in §4. The paradigm's real value is not the CLI; it is the
**artifact discipline**: every feature passes through what → how → task-list
gates, each artifact reviewable before the next phase spends tokens.

### mattpocock/skills is "small composable skills over monolithic instructions"

Not "traits" or "skill trees" — it is a `skills/` tree of `SKILL.md` files in
exactly the format Claude Code already uses, split into **user-invoked**
workflow orchestrators (`/grill-me`, `/tdd`, `/implement`, `/handoff`) and
**model-invoked** disciplines the agent reaches for automatically. The
philosophy, distilled:

1. Small, composable, adaptable skills beat one big instruction file.
2. "No-one knows exactly what they want" — alignment interviews (*grilling*)
   before implementation, which is spec-kit's `/clarify` by another name.
3. Feedback loops (types, tests) prevent silent failure — the agent should be
   caught by tooling, not by vigilance.

The two references converge on the same shape: **expensive-model judgment
gets encoded once into artifacts (specs) and procedures (skills);
cheap-model execution is bounded by deterministic checks.**

### What actually lets Haiku replace Fable

From both sources plus first principles, the properties that matter:

| Property | Mechanism |
|---|---|
| Small blast radius | tasks.md units — one task, one verifiable outcome |
| Deterministic guardrails | CI/tests/lints that *fail*, not prose that advises |
| Ambiguity has an exit | `[NEEDS CLARIFICATION]` markers; gates that stop the pipeline |
| Context is scoped | Skill/spec loaded per task, not a 2,000-line brain dump |
| Review at phase edges | Human/Fable reviews spec.md and plan.md — cheap to read, before tokens are spent on code |

The single biggest lever is the second one, and it is tooling work, not
prompt work. Prose rules ("no `unwrap()` in library code") depend on the
model noticing; a lint that fails the build does not.

---

## 2. Audit of the current setup

### What exists

```
AGENTS.md                          ~100 lines, canonical constraints — GOOD, is already a constitution in prose
CLAUDE.md (root)                   pointer to AGENTS.md + graphify rules — fine
.claude/
  CLAUDE.md                        graphify skill pointer — fine
  settings.json                    permissions allowlist + graphify PreToolUse/PostToolUse hooks — VALUABLE, keep
  settings.local.json              local overrides — keep
  skills/graphify/                 actively used (hooks reference it) — keep
  adrs/ (ADR-001..004)             decision history — keep, feeds the constitution
  plans/2026-07-12-strategic...    active roadmap — keep, becomes the spec backlog
.github/
  workflows/{rust,docs,models,audit}.yml   real CI (lint, tests, TypeSpec compile, cargo-audit) — keep
  demo.tape, demo.gif              VHS demo — keep
  agents/{context7,rust-expert,documentation-writer}.agent.md   VS Code Copilot custom agents (463/150/63 lines)
  instructions/{rust-best-practices,markdown,no-heredoc}.instructions.md   Copilot path-scoped rules
  prompts/documentation-writer.prompt.md
  skills/{adr,prd,refactor}/SKILL.md        (166/142/644 lines)
```

### Assessment

- **AGENTS.md is the strongest asset.** The crate-boundary table, MSRV, error
  stratification, and TEA invariant are constitution articles already — they
  just lack (a) numbering/versioning and (b) mechanical enforcement. Verdict:
  it does not become YAML; it becomes the **constitution**, with each article
  annotated by *how it is enforced* (CI job, lint, test, or "review-only").
- **ADRs are decisions, not specs — keep them as-is.** They record *why*;
  specs record *what/how next*. ADR-005 (authoring editor scope) should be
  the first decision produced *through* the new pipeline: `/specify` output
  becomes the ADR's context section.
- **The `.github/` agent stack is a parallel universe.** The three
  `.agent.md` files, three `.instructions.md`, one prompt, and three skills
  are Copilot-specific (~1,864 lines total). They are decently built —
  `rust-best-practices.instructions.md` correctly *delegates* to AGENTS.md
  rather than duplicating it — but:
  - `prd/SKILL.md` is superseded outright by `/specify` (a spec.md *is* the
    PRD in this paradigm).
  - `refactor/SKILL.md` (644 lines) is generic, imported content — exactly
    the "monolithic instruction" mattpocock's approach argues against.
  - `context7.agent.md` (463 lines) duplicates what the context7 MCP server
    instructions already enforce in Claude Code.
  - **Open decision:** are the Copilot artifacts in active use in VS Code?
    If yes, keep the thin ones (`instructions/`) and delete the heavy ones.
    If no, delete `agents/`, `prompts/`, and `skills/{prd,refactor}`
    entirely; move `skills/adr` under `.claude/skills/`.
- **Skills today are procedures, not specs.** graphify/verify/code-review are
  *how to do a recurring job* — they stay skills. Specs are *what a feature
  must do* — per-feature, disposable-ish artifacts. Keeping the two concepts
  separate avoids the category error in the original brief (skills are not
  "specs implemented in the harness").
- **The enforcement gap is the real finding.** Of AGENTS.md's hard rules,
  today only fmt/clippy/tests run in CI. Not machine-checked: crate
  boundaries, MSRV, no-unwrap, missing-docs, validator parity. Every one of
  these is mechanizable (§3C).

---

## 3. Proposed design: SpecKit-shaped, natively implemented

### A. Artifact & spec taxonomy

| Artifact | Role | Written by | Executed/enforced by | Fails safely via |
|---|---|---|---|---|
| `AGENTS.md` (constitution) | Immutable principles, numbered articles, each with an Enforcement line | Human + Fable, rarely | CI + lints where possible; plan-gate checklist otherwise | Build failure or plan-gate stop |
| `specs/NNN-*/spec.md` | What/why: user stories, acceptance criteria, out-of-scope | Fable (or human) via `/specify` | Reviewed by human before `/plan` | `[NEEDS CLARIFICATION]` blocks progression |
| `specs/NNN-*/plan.md` | How: crates touched, spec-first protocol changes, test plan, constitution check | Fable via `/plan` | Reviewed; constitution-check section is a hard gate | Gate answers "violates Article N" → stop |
| `specs/NNN-*/tasks.md` | Ordered small tasks, each with a verification command | Fable via `/tasks` | **Haiku** via `/implement` | Each task ends with a runnable check (test/clippy/tmux smoke) |
| `.claude/adrs/*.md` | Decision history | Human + model | Referenced by constitution | n/a (history) |
| Protocol TypeSpec (`protocol/main.tsp`) | Wire-format spec | Already exists | Schema compile in CI (`models.yml`), dual validators | Already enforced — this is the house's existing SDD, extended not replaced |
| Fixture corpus (`protocol/fixtures/`) | Validator-parity spec | From strategic plan | Both test suites | Test failure |

Deliberately **not** specced: heuristics ("prefer small functions"), taste,
and anything the type system or a test can't observe. Over-specification is
the failure mode where the constitution becomes prose soup again.

### B. File reorganization

```
AGENTS.md                     → restructured as numbered constitution (content ~same, + Enforcement lines, + version/date)
specs/                        → NEW, repo root: NNN-feature-name/{spec,plan,tasks}.md
.claude/skills/specify/       → NEW skill (what/why interview → spec.md; grilling per mattpocock)
.claude/skills/plan/          → NEW skill (spec.md → plan.md; runs constitution check)
.claude/skills/tasks/         → NEW skill (plan.md → tasks.md with per-task verify commands)
.claude/skills/implement/     → NEW skill (Haiku's runbook: one task at a time, verify, stop on ambiguity)
.claude/skills/adr/           → MOVED from .github/skills/adr
.github/skills/prd/           → DELETE (superseded by /specify)
.github/skills/refactor/      → DELETE (generic; /simplify + /code-review cover it)
.github/agents/, prompts/     → DELETE if Copilot agents unused (pending decision); else slim
.github/instructions/         → fold no-heredoc + markdown rules into AGENTS.md appendix if Copilot unused
SDD.md (or CONTRIBUTING §)    → NEW, one page: the pipeline, who runs what, how to read a spec dir
```

Rationale for **not** installing spec-kit's own scaffolding: `specify init`
would add a Python/uv toolchain dependency and `.specify/` templates written
for greenfield generic projects, whose constitution template we would
immediately gut and replace with AGENTS.md content. We adopt the artifact
pipeline and gate discipline; we implement it as four small project skills in
the harness already in use. If spec-kit's templates improve later, they can
be diffed in — the artifact names and layout are kept compatible
(`specs/NNN-name/spec|plan|tasks.md`) precisely so that migration stays open.

### C. The guardrails that make Haiku viable (tooling, week 1)

1. **Crate-boundary check** — a workspace test (or `cargo xtask check-boundaries`)
   that parses `cargo metadata` and asserts each crate's dependencies against
   the constitution's allowlist table. Today's prose table becomes data.
2. **MSRV in CI** — `cargo msrv verify` job (the 1.88 promise is currently untested).
3. **No-unwrap as lint** — `[lints.clippy] unwrap_used = "deny", expect_used = "deny"`
   in the three library crates (with targeted `#[allow]` for tests/LazyLock),
   turning AGENTS.md's top idiom into a build failure.
4. **Missing docs** — `#![deny(missing_docs)]` (or lint-table equivalent) per
   library crate, mechanizing the doc-comment rule.
5. **Validator parity fixtures** — already specced in the strategic plan;
   doubles as the protocol-conformance spec here.

After these land, the constitution articles Haiku most plausibly violates all
fail loudly in `cargo test`/`clippy` — locally, before CI. That, more than
any spec format, is what "fail safely" means.

### D. Escalation protocol (executor story)

- `/implement` skill instructs: work one task, run its verify command, commit
  or report; on *any* mismatch between task text and code reality, or any
  `[NEEDS CLARIFICATION]` remnant, **stop and ask** (AskUserQuestion / return
  to the human) rather than improvise.
- Constitution check in plan.md is a table: Article → touched? → compliant?
  A "no" is a stop, not a judgment call.
- Specs carry `date:` and `status:` frontmatter; a stale `in-progress` spec
  older than the current branch's base is a prompt to re-verify, addressing
  spec rot. Completed specs get `status: shipped` and become history (they do
  not need maintenance — the constitution and tests are the living parts).

### E. Dangers

- **Over-specification:** cap spec.md at what a reviewer reads in 5 minutes;
  push detail into plan.md; push mechanics into tasks.md. If a rule can't be
  enforced or reviewed, don't write it.
- **Spec rot:** specs are per-feature and *retired at ship*; only the
  constitution, skills, and tests persist. Rot surface stays small by design.
- **Pipeline overhead on small changes:** not every change is a feature.
  Bug fixes and chores skip the pipeline; the threshold ("would this need an
  ADR or touch the protocol or take >1 day?") goes in SDD.md.
- **Executor confusion:** if Haiku misreads a good spec, the fix is a better
  tasks.md granularity, not more prose — feed the failure back into the
  `/tasks` skill template.

---

## 4. Three options, ranked

**Option B — SpecKit-inspired, native skills + mechanical guards (RECOMMENDED)**
- Haiku-reliability: **High.** Small tasks + hard lints + stop-on-ambiguity.
- Retrofit cost: **Medium.** ~4 new skills, constitution restructure, 4 CI/lint
  changes, deletions. No new toolchain.
- Maintenance: **Low.** Persistent surface is constitution + 4 skills; specs retire.
- mattpocock alignment: **High** — literally his format and philosophy.

**Option A — Adopt spec-kit wholesale (`specify init --here`)**
- Haiku-reliability: High for the pipeline, but the guardrail work (§3C) is
  still on us — spec-kit ships none of it.
- Retrofit cost: Medium-high. uv/Python dependency, generic templates to
  rewrite (its constitution template vs. AGENTS.md), greenfield assumptions,
  upstream template churn to track.
- Maintenance: Medium. `.specify/` scripts/templates are third-party surface.
- Verdict: buys the same artifacts at higher cost; sensible only if you want
  the multi-agent portability (Copilot + Claude both driving the same specs)
  as a first-class requirement today.

**Option C — Status quo + documentation**
- Haiku-reliability: Low-medium. Haiku with today's AGENTS.md still relies on
  prose vigilance; no phase gates; features remain expensive-model work.
- Cost/maintenance: near zero / unchanged.
- Verdict: rejected — it leaves the stated goal (medium-cost execution) unmet,
  and leaves ~1,700 lines of partially-rotted `.github/` agent content in place.

---

## 5. Recommendation

**Option B.** Fireside is unusually well-positioned: the protocol is already
spec-first (TypeSpec → generated schemas → dual validators → canonical
example), AGENTS.md is already constitution-shaped, and the test culture is
real. The missing layers are (1) mechanical enforcement of the constitution
and (2) a feature pipeline with reviewable gates. Both are cheap to add
natively; neither is provided by spec-kit's tooling. Keep spec-kit's artifact
names and layout for future compatibility, skip its CLI.

Division of labor after the retrofit:
- **Fable/Opus:** `/specify`, `/plan`, `/tasks`, constitution changes, ADRs,
  review at phase gates.
- **Haiku:** `/implement` task-by-task, `/verify`, graphify updates, docs
  drafts against a spec, fixture authoring from a spec table.

## 6. Two-week implementation roadmap

**Week 1 — guardrails + scaffolding (mostly mechanical; Haiku can execute much of it)**
1. Decide the Copilot question (`.github/agents` in use or not) — 5-minute
   human decision that gates the deletions.
2. Land §3C items 1–4: boundary-check test, `cargo msrv verify` CI job,
   unwrap/expect lints, missing-docs lints. Fix whatever they flush out.
3. Restructure AGENTS.md into numbered articles with Enforcement lines;
   root CLAUDE.md keeps pointing at it (unchanged behavior for every agent).
4. Create `specs/` + the four skills (`specify`, `plan`, `tasks`,
   `implement`) — written once by Fable, each under ~150 lines, mattpocock
   style. Write SDD.md (one page).
5. Execute deletions/moves from §3B; run `graphify update .`.

**Week 2 — pilot + calibrate**
6. Pilot the full pipeline on **`fireside validate --watch`** (P0, small,
   well-understood — ideal first spec): Fable runs `/specify`→`/plan`→`/tasks`,
   human reviews each artifact, **Haiku runs `/implement`** end to end.
7. Measure: how many tasks Haiku completed without escalation; where it
   stopped; whether guardrails caught anything. Feed fixes into the skill
   templates.
8. Second pilot: ADR-005 (authoring scope) drafted through `/specify` +
   `/adr` — the decision the strategic plan already requires.
9. Add the fixture-parity corpus (strategic plan Week-1 item) as
   `specs/002-validator-parity/` — dogfooding the pipeline on protocol work.

Exit criteria: one feature shipped where Fable wrote ≤3 markdown artifacts,
Haiku wrote all the code, and no constitution article was violated silently.

## Unknowns flagged

- spec-kit has **no published brownfield methodology** — §3B/§6 is our own
  synthesis; revisit upstream in a few months for a `specify init --here`
  migration path.
- Whether the VS Code Copilot artifacts are load-bearing for the user's
  workflow — blocking decision for the deletion list.
- `cargo msrv verify` runtime in CI (may want it on a schedule rather than
  every PR if slow).
- Exact clippy lint-table syntax interaction with the 2024-edition workspace
  lints table — verify on a branch before landing repo-wide.
