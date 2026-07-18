# Fireside Docs

This directory contains the Fireside Protocol specification site built with Astro and Starlight.

## Stack

- Astro 5.17
- Starlight 0.32
- astro-mermaid (package-based Mermaid integration)

## Content Structure

- `src/content/docs/index.md` — landing page
- `src/content/docs/spec/` — 6 normative chapters + 3 non-normative appendices
- `src/content/docs/guides/` — 3 user guides: `getting-started` (build a small
  graph by hand), `authoring-markdown` (compile a talk with `fireside import`),
  `presenting` (every key the TUI responds to)
- `src/content/docs/reference/` — `cli` (every subcommand/flag/exit code),
  `data-model-quick-reference`, `domain-vocabulary`, `conformance`

Sidebar groups are configured explicitly in `astro.config.mjs` (not
`autogenerate`), so a new page under any of these directories also needs a
matching entry added to the sidebar there.

Architecture Decision Records live in `.claude/adrs/` at the repo root — they're
project-history artifacts for maintainers/AI agents, not published on this
site.

## Local Development

```bash
npm install
npm run dev
```

Site URL: `http://localhost:4321/fireside/`

## Validate and Build

```bash
npm run check   # Astro diagnostics
npm run build    # Full build with type checking
```
