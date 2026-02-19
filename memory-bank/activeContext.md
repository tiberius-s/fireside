# Active Context

## Current Focus

Execution of `.github/prompts/plan-fireside-tui-ux-initiative.prompt.md`
across phased milestones, while keeping protocol/docs aligned.

Current execution mode is TUI usability first:

- JSON-first configuration surfaces (`fireside.json`) for project config.
- No YAML/TOML expansion work for now unless explicitly re-approved.
- Export workflows (HTML/PDF) are deferred from active implementation scope.
- Competitive analysis is tracked in memory-bank, not user-facing docs.

## Recently Applied Direction

- Replaced `group` with `container` in protocol model and docs.
- Replaced `x-` prefix extension convention with explicit extension blocks:
  `kind: "extension"` + `type`.
- Standardized serialization guidance to `application/json`.
- Removed root `specs/` duplication by moving quick-reference docs into
  `docs/src/content/docs/reference/`.
- Enforced chapter ordering in docs sidebar: §1–§6 then appendices.

## Next Workstream

- Continue Phase 5 and Phase 6 TUI slices from the UX initiative:
  Mermaid pipeline hardening, settings integration, and hot-reload polishing.
- Keep task tracking explicitly phase-aligned so progress is easy to audit.

## Current Milestone Execution

- Implemented runtime handling for `traversal.after` in engine traversal to
  support branch rejoin behavior.
- Implemented project-directory edit support and shared project entry
  resolution using `fireside.json`.
- Next milestone slices: Mermaid output path hardening, hot-reload, and
  release polish that directly improves TUI usability.
