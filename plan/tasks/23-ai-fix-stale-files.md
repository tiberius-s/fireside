# Task 23 — AI workflow: fix stale agent/instruction files

**Depends on:** none (do after 22 to avoid merge conflicts in the same files)
**Crates:** none (AI config only)
**Phase:** any (findings A2, A3, A4)

## Goal

Three hygiene fixes: stale Context7 tool name, self-contradicting markdown rules, and a planning prompt stranded in a directory no tool reads.

## Steps

1. **context7.agent.md (A2)** — `.github/agents/context7.agent.md`:
   - Replace every `mcp_context7_get-library-docs` with the current tool `mcp_context7_query-docs` (compare with rust-expert.agent.md, which is correct: `resolve-library-id` → `query-docs`).
   - Replace the Express/React/Tailwind examples with this repo's stack: ratatui, serde, clap, TypeSpec, Astro/Starlight.
   - Soften "ALWAYS inform users about upgrades" to: mention upgrades only when the user asks about versions or a verified API differs from the pinned version.
2. **markdown.instructions.md (A3)** — `.github/instructions/markdown.instructions.md`:
   - Resolve the contradiction (rule 7 says 400-char lines; Formatting says 80): defer to `.markdownlint.json` as the single source of truth — read it first and state its actual MD013 setting instead of either number.
   - Remove the `csharp` example (use `rust`), and align the H1 guidance with reality: docs-site content (`docs/src/content/docs/**`) gets titles from frontmatter (no H1); repo-root markdown may use H1.
3. **Planning prompt location (A4)** — `.claude/prompts/claude-fable-plan.prompt.md` is read by nothing (Copilot reads `.github/prompts/`; Claude Code reads `.claude/commands/`). Pick its audience and move it:
   - if Copilot: move back to `.github/prompts/`;
   - if Claude Code: convert to `.claude/commands/audit-roadmap.md` — strip the Copilot `tools:`/`agent:` frontmatter (keep `description:`), so `/audit-roadmap` invokes it.
   - Either way, remove the empty `.claude/prompts/` directory.

## Do NOT

- Rewrite the agents' substance (Task 22 handled the rule consolidation).
- Add markdownlint to CI here (optional follow-up; note it in the PR description).

## Acceptance

```bash
grep -rn "get-library-docs" .github/ | wc -l        # 0
grep -n "400" .github/instructions/markdown.instructions.md | wc -l   # 0
ls .claude/prompts 2>&1                              # No such file or directory
```
