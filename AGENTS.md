# Fireside — Agent Guide

**The canonical engineering rules live in the project constitution:
[`.specify/memory/constitution.md`](.specify/memory/constitution.md).
Read it before writing any code.** It covers the seven core principles:
spec-as-source-of-truth, presenter-first UX, crate boundaries, mandatory
idioms, error stratification, MSRV 1.88, and test discipline. If any other
document disagrees with the constitution, the constitution wins.

## Spec-driven workflow

Features go through the Spec Kit pipeline (installed for both Claude Code
and GitHub Copilot):

1. `/speckit-specify` — write `specs/NNN-feature/spec.md` (what/why)
2. `/speckit-clarify` — optional; resolve ambiguities before planning
3. `/speckit-plan` — write `plan.md` (how); includes the Constitution Check gate
4. `/speckit-tasks` — write `tasks.md` (ordered, verifiable task units)
5. `/speckit-implement` — execute tasks one at a time, verifying each

Bug fixes and mechanical chores may skip the pipeline. Anything touching
the wire format needs a spec change and an ADR (`.claude/adrs/`) first.

## Everyday commands

- `cargo test --workspace` — full test suite
- `cargo clippy --workspace --all-targets` — lints (keep silent)
- `node protocol/validate.mjs <file>` — validate a document
- `cd protocol && npm run build` — regenerate schemas from TypeSpec;
  commit `tsp-output/` (CI enforces this)
- `npm run check --prefix docs` — docs site check
- `graphify update .` — refresh the knowledge graph after code changes

## Before handing off any change

Run `scripts/verify.sh` (add `--skip-slow` for a faster inner-loop pass,
but run the full version at least once before calling work done). It runs
every check every CI workflow actually runs, job for job — not just the
commands above. This exists because those two things have drifted before:
a `main.tsp` doc-comment-only edit changed generated JSON Schema
`description` text and got pushed without a `tsp-output/` rebuild, which
only CI caught. Two gotchas the script encodes so you don't have to
remember them:

- **Any edit to `protocol/main.tsp`, including doc comments, requires
  `cd protocol && npm run build` before committing** — doc comments flow
  into the generated schemas' `description` fields, so even a
  comment-only change can drift `tsp-output/` out of sync.
- **GitHub Actions workflow files are not named what they say they are.**
  `.github/workflows/models.yml`'s `name:` field is `Protocol` — it shows
  up in the Actions tab and `gh run list` as "Protocol", not "models". Use
  `gh api repos/<owner>/<repo>/actions/workflows` (or just read the
  `name:` field in each file) rather than guessing from filenames when
  investigating a failed run.
