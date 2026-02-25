---
name: adr
description: 'Generate Architecture Decision Records (ADRs) for the Fireside project.'
---

# ADR Skill

Generate Architecture Decision Records (ADRs) for the Fireside project.

## When to Use

Invoke this skill whenever a significant architectural decision needs to be captured:

- Choosing or replacing a major crate dependency (e.g., switching test runner, parser, renderer)
- Making a protocol-breaking or protocol-additive change that has long-term design implications
- Establishing or changing a cross-cutting convention (e.g., error handling strategy, serde tagging)
- Adding or removing a crate from the workspace
- Changes to CI infrastructure or toolchain requirements

**Do not** write ADRs for routine implementation choices, bug fixes, or documentation updates.

---

## ADR Format (Nygard-style)

Every ADR uses this exact structure:

```markdown
---
title: 'ADR-[NNN]: [Short Title]'
status: 'proposed | accepted | deprecated | superseded'
date: 'YYYY-MM-DD'
deciders: ['@handle1', '@handle2']
---

# ADR-[NNN]: [Short Title]

## Status

[proposed | accepted | deprecated | superseded by ADR-NNN]

## Context

[1–3 paragraphs. Describe the situation, the forces at play, the constraints that make this a non-trivial decision. Be concrete — name the actual crates, actual files, actual behavior. Avoid abstract language.]

## Decision

[1–2 paragraphs. State what was decided in plain terms. Use active voice: "We will use X because Y." Do not hedge.]

## Consequences

### Positive

- [Concrete benefit 1]
- [Concrete benefit 2]

### Negative or Trade-offs

- [Concrete cost or limitation 1]
- [Concrete cost or limitation 2]

### Neutral / Follow-up

- [Work that this decision creates or defers]
```

---

## Numbering and Filing

1. **Find the next ADR number**: Look in `docs/src/content/docs/explanation/` for any existing `adr-*.md` files, or query the user if unsure. ADRs are numbered sequentially with zero-padded three digits: `ADR-001`, `ADR-002`, etc.

2. **File location**: `docs/src/content/docs/explanation/adr-NNN-short-title.md`
   - File names use kebab-case: `adr-001-chose-ratatui.md`
   - One ADR per file. Never append to an existing ADR.

3. **Sidebar registration**: After creating the file, check `docs/astro.config.mjs` and add the new ADR to the `explanation` section of the sidebar under a collapsible "Architecture Decisions" group. Follow the existing manual ordering pattern.

---

## Writing Guidelines

### Context section

Answer these questions (not as headers — prose form):

- What problem are we solving?
- What options were considered?
- What constraints (MSRV, crate boundary rules, protocol stability) apply?
- What happens if we do nothing?

### Decision section

- State the chosen option once, clearly.
- Include the rationale: why this option over the alternatives.
- If a crate is referenced, include the crate name, version, and a brief description of what it provides.

### Consequences section

Be honest. Every significant decision has trade-offs. If you can only think of positives, you haven't thought hard enough.

---

## Project-Specific Constraints

When writing ADRs for Fireside, always consider:

| Constraint         | Rule                                                                                          |
| ------------------ | --------------------------------------------------------------------------------------------- |
| Crate boundaries   | `fireside-core` must not gain I/O or UI deps. Check `copilot-instructions.md` boundary table. |
| Protocol stability | `0.1.x` changes must be additive. Any breaking change requires a version bump.                |
| MSRV               | MSRV is **1.88**. Proposed dependencies must be MSRV-compatible.                              |
| Wire format        | All JSON property names are kebab-case. No exceptions.                                        |
| TEA invariant      | `App::update` is the sole mutation point in `fireside-tui`.                                   |

---

## Example ADR

```markdown
---
title: 'ADR-001: Use cargo-nextest as the primary test runner'
status: 'accepted'
date: '2024-11-15'
deciders: ['@tiberius']
---

# ADR-001: Use cargo-nextest as the primary test runner

## Status

Accepted

## Context

The Fireside workspace has tests spread across four crates. `cargo test` runs all tests in
a single process and does not provide per-test timing or parallel test isolation. As the
test suite grows, slow tests can block the quality gate without any visibility into which
test is the bottleneck.

`cargo nextest` is a drop-in replacement with per-test isolation, parallel execution, and
structured output. It is already installed in CI.

## Decision

We will use `cargo nextest run --workspace` as the primary test command in all documentation,
CI configuration, and copilot instructions. `cargo test --workspace` remains as a documented
fallback for environments where nextest is not installed.

## Consequences

### Positive

- Per-test timing output makes regressions visible immediately.
- Parallel test isolation prevents shared-state test interference.
- Structured output integrates cleanly with CI problem matchers.

### Negative or Trade-offs

- Requires `cargo install cargo-nextest` in new development environments.
- Not available in `cargo test`-only CI runners without a setup step.

### Neutral / Follow-up

- `.githooks/install.sh` should be updated to check for nextest and print an install hint if missing.
```
