---
description: 'Generate a concrete improvement plan for the Fireside repo by auditing the protocol spec, verifying the Rust reference implementation, and proposing a token-efficient execution roadmap.'
---

# Claude Fable Planning Prompt

Use this prompt with Claude Fable to produce a practical, evidence-based plan for improving this repository.

## Goal

Produce a plan that helps this repository shine by:

1. Auditing the protocol spec and identifying any real gaps, ambiguities, or drift.
2. Evaluating the current Rust reference implementation and the TUI/CLI UX.
3. Recommending a phased roadmap that is realistic for a smaller, more token-efficient model such as Sonnet or MAI Code 1 Flash.
4. Grounding Rust recommendations in the repository’s existing best practices, crate boundaries, and maintainability constraints.

## Repository Context

This repository contains:

- the normative protocol definition in `protocol/main.tsp`
- generated JSON Schema under `protocol/tsp-output/schemas/`
- documentation in `docs/src/content/docs/spec/`
- the Rust reference implementation in `crates/`

Treat the docs and the TypeSpec source as primary evidence. Do not rely on memory or assumptions.

When the work touches Rust code, also consult:

- `.github/agents/rust-expert.agent.md` for crate-boundary, MSRV, and Context7 verification guidance.
- `.github/instructions/rust-best-practices.instructions.md` for modularity, readability, and maintainability expectations.

If a library or crate API is involved, use Context7 to verify the current API before recommending it.

## Required Work

### Part A — Audit the protocol spec and find gaps

Start with the docs and the protocol source. Specifically:

- Read the spec chapters under `docs/src/content/docs/spec/`.
- Read `protocol/main.tsp` and compare it to the generated schemas and the Rust implementation.
- Identify any gaps, drift, or contradictions between:
  - the documented spec,
  - the TypeSpec source,
  - the generated JSON Schema,
  - the Rust runtime model in `crates/fireside-core/` and `crates/fireside-engine/`.

Look especially for:

- missing normative behavior that is implied by the implementation but not documented,
- documented behavior that is no longer true in the code,
- unsupported or inconsistent content block variants,
- traversal semantics that are under-specified or mismatched,
- any schema/implementation drift that would confuse authors or tool builders.

When you find a potential gap, do not just list it. Explain:

- why it matters,
- what evidence supports it,
- which part of the repo currently reflects the mismatch,
- and what concrete change would close the gap.

### Part B — Evaluate the Rust reference implementation and UX

Now move into `crates/` and study the current implementation status.

Use the shell interactively where appropriate. Run commands such as:

- `cargo test --workspace`
- `cargo test -p fireside-engine`
- `cargo test -p fireside-tui`
- `cargo run -p fireside-cli -- validate docs/examples/hello.json`
- `cargo run -p fireside-cli -- present docs/examples/hello.json`

If the UI can be launched in your environment, explore the actual TUI behavior. If not, use the tests, CLI output, and source inspection to reason about the UX.

Focus on:

- what is already working well,
- what is currently broken or incomplete,
- where the user experience feels weak or confusing,
- what is over-complex for the current scope,
- what would make the implementation easier to maintain and easier to reason about,
- whether the proposed work keeps crate responsibilities modular and testable.

You should explicitly assess:

- the CLI entry points,
- the engine/traversal logic,
- the TUI rendering and interactions,
- the editor workflow,
- any evidence of missing spec support or runtime bugs.

### Part C — Produce a concrete, phased plan

Your final output should be a practical roadmap for improving the repository.

Structure the response as:

1. A short summary of the repo’s current state.
2. A list of spec gaps and drift you found, with evidence and suggested fixes.
3. A progress assessment of the Rust reference implementation by crate.
4. A UX assessment of the current CLI/TUI experience.
5. A prioritized plan for the next 3–6 weeks of work.
6. A token-efficient implementation strategy for a smaller model.

The plan should be actionable and should favor small, well-bounded steps over large rewrites.

## Output Expectations

Be concrete, evidence-based, and concise.

- Cite actual files and commands when possible.
- Distinguish between "spec gap", "implementation bug", and "UX issue".
- For any Rust recommendation, explicitly verify crate/API choices with Context7 and explain how they fit the existing Fireside boundaries and MSRV.
- For any implementation plan, call out the maintainability, testability, and modularity trade-offs, not just the feature outcome.
- Prefer a plan that is implementable in small iterations.
- Emphasize what should be fixed first, what can wait, and why.
- If something is uncertain, say so explicitly instead of pretending confidence.

## Important Constraints

- Do not invent solutions that are not supported by the code or docs.
- Do not claim test success without running the relevant command.
- Use the shell and the repo itself as your primary sources of truth.
- The final plan should be strong enough that a smaller model could execute it efficiently.
