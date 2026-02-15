# Fireside Docs

This directory contains the Fireside Protocol specification site built with Astro and Starlight.

## Stack

- Astro 5.17
- Starlight 0.32
- astro-mermaid (package-based Mermaid integration)

## Content Structure

- `src/content/docs/index.md` — landing page
- `src/content/docs/spec/` — 6 normative chapters + 3 non-normative appendices
- `src/content/docs/schemas/` — schema reference pages (graph, node, content-blocks)
- `src/content/docs/guides/` — 3 user guides (getting-started, branching-adventures, for-designers)
- `src/content/docs/decisions/` — Architecture Decision Records (ADRs)

Sidebar groups are configured in `astro.config.mjs` using `autogenerate` so newly added
pages under these directories are picked up automatically.

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
