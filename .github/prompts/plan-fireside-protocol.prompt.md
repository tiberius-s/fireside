# Fireside Protocol 0.5.0-draft — Comprehensive Revision Plan

> Everything written after the audit and DDD exercise supersedes 0.4.0-draft, no matter how drastic the change.
> Rust code is out of scope — it will be re-done to mirror the specification/protocol.

## Guiding Principle

**"A portable format for branching presentations and lessons."**

## Mental Models

- **Museum tour**: Rooms (nodes) connected by doorways (edges). Some rooms have interactive kiosks (branch points). A map (graph) shows the whole floor. You can wander freely or follow a guided path.
- **Subway map**: Stations (nodes) on lines (sequences). Transfer stations (branch points) let you switch lines. The system map (graph) shows all routes. You always know where you are and where you can go.

## Brand Identity

- **Protocol Name:** Fireside
- **Theme:** Hearth / fireside chat — warmth, storytelling, gathering
- **Two-Layer Vocabulary:**
  - **Technical layer** (spec, schema, TypeSpec): Graph, Node, Traversal, BranchPoint, BranchOption, ContentBlock
  - **Brand layer** (guides, marketing, landing page): Session, Moment, Flow, Question, Answer, Card
- **Aesthetic direction:** Warm, inviting, lo-fi. Campfire orange/amber palette for docs. Storytelling voice in guides.

## 8 Canonical User Journeys

| ID  | Persona            | Scenario                                     |
| --- | ------------------ | -------------------------------------------- |
| A   | Teacher            | Interactive lesson with branching quiz paths |
| B   | Security trainer   | Phishing escape room with consequences       |
| C   | Product manager    | Demo with audience-driven feature deep-dives |
| D   | Museum curator     | Kiosk with self-guided exhibit exploration   |
| E   | Tabletop GM        | Campaign session with player choices         |
| F   | Developer advocate | Onboarding tutorial with skill-check gates   |
| G   | Therapist          | Guided reflection with branching prompts     |
| H   | Parent             | Bedtime story with child-driven choices      |

---

## Phase 0 — Brand Identity Foundation

- Rename protocol: Hyphae → **Fireside**
- Define two-layer vocabulary architecture (technical + brand)
- Establish aesthetic direction (warm, storytelling, lo-fi)
- Define mental models (museum tour, subway map)
- Map 8 user journeys to protocol capabilities

## Phase 1 — DDD Starter Modelling Process

Apply the [DDD Starter Modelling Process](https://github.com/ddd-crew/ddd-starter-modelling-process) (8 steps):

1. **Align** — Big picture: branching content navigation protocol
2. **Discover** — Domain events: GraphLoaded, NodeEntered, BranchChosen, HistoryPushed, TraversalComplete
3. **Decompose** — Bounded contexts: Graph (data model), Traversal (navigation logic), Rendering (presentation), Validation (integrity)
4. **Connect** — Context map: Graph ↔ Traversal (shared kernel), Traversal → Rendering (downstream), Graph → Validation (downstream)
5. **Strategize** — Core domain: Graph + Traversal. Supporting: Validation. Generic: Rendering, Design System
6. **Organize** — Team topology: spec authors own Graph+Traversal+Validation; implementors own Rendering+Design System
7. **Define** — Bounded context canvases for each context
8. **Code** — TypeSpec model as the ubiquitous language implementation

## Phase 2 — Spec Structure Revision

### Current (10 chapters — too closely mirrors GraphQL spec)

§1 Overview, §2 Type System, §3 Operations, §4 Execution, §5 Validation, §6 Rendering, §7 Design System, §8 CLI, §9 Security, §10 Extensibility

### New (6 normative chapters + 3 non-normative appendices)

**Normative chapters:**

| Ch  | Title         | Covers                                                                                                                                     |
| --- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| §1  | Introduction  | Conformance, terminology, design principles, notation conventions                                                                          |
| §2  | Data Model    | Graph, Node, ContentBlock (7 core), BranchPoint, BranchOption, Traversal — derives from TypeSpec                                           |
| §3  | Traversal     | Four operations (Next, Choose, Goto, Back) as numbered imperative steps. History stack contract. State machine.                            |
| §4  | Validation    | JSON Schema 2020-12 rules, graph integrity (reachability, ID uniqueness, dangling refs), structural validation                             |
| §5  | Extensibility | `x-` prefix extension model (OpenAPI-inspired), vocabulary system (JSON Schema-inspired), fallback rendering contract, version negotiation |
| §6  | Serialization | JSON primary format, YAML authoring alternative, `$schema` self-describing documents, media types, file extensions                         |

**Non-normative appendices:**

| App | Title                   | Covers                                                                                                                                                                 |
| --- | ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A   | Design System           | Token architecture, theme resolution, spacing scale, WCAG compliance — advisory, not required for conformance                                                          |
| B   | Engine Guidelines       | TEA guarantees (immutable graph, sequential ops, history stack, stateless render, visitor pattern for CRUD), mode transitions — prescribes guarantees not architecture |
| C   | Content Block Reference | Full catalog of 7 core blocks + extension examples, rendering hints, accessibility guidance                                                                            |

**Removed from spec (implementation-specific):**

- CLI (§8) — implementation decision, not protocol
- Security (§9) — becomes a section within relevant chapters
- Rendering details (§6) — absorbed into Appendix B + C

## Phase 3 — Navigation → Traversal Rename

Full wire-format rename:

| Old                           | New                          |
| ----------------------------- | ---------------------------- |
| `Navigation` (type)           | `Traversal` (type)           |
| `navigation` (field on Node)  | `traversal` (field on Node)  |
| `Navigation` (TypeSpec model) | `Traversal` (TypeSpec model) |
| All spec references           | Updated throughout           |
| All guide references          | Updated throughout           |
| All schema doc pages          | Updated throughout           |

## Phase 4 — Content Block Redesign

### Current: 11 content block types

heading, text, code, list, image, divider, blockquote, table, fragment, spacer, columns

### New: 7 core + extension model

**Core blocks (MUST be supported by conforming engines):**

| Block     | Purpose                                                                                |
| --------- | -------------------------------------------------------------------------------------- |
| `heading` | Section headings (level 1-6, constrained by @minValue/@maxValue)                       |
| `text`    | Prose content (Markdown subset)                                                        |
| `code`    | Source code with language hint and optional line highlighting                          |
| `list`    | Ordered or unordered lists                                                             |
| `image`   | Images with alt text, source, optional caption                                         |
| `divider` | Visual separator                                                                       |
| `group`   | Generic container (replaces fragment + columns). Layout hint property for arrangement. |

**Removed from core:**

- `spacer` → rendering concern, not content (use CSS/theme spacing)
- `fragment` → renamed to `group` (generic container)
- `columns` → subsumed by `group` with `layout: "columns"` hint
- `table` → moved to extension (`x-table`)
- `blockquote` → moved to extension (`x-blockquote`)

**Extension model:**

- `x-` prefix for custom block kinds (e.g., `x-table`, `x-video`, `x-poll`)
- `x-` prefix for custom properties on any block
- Fallback rendering contract: engines MUST render unknown `x-` blocks as their `fallback` property (text or group)
- Vocabulary system for future standardization of extensions

## Phase 5 — TypeSpec Model Revision

### Changes to `models/main.tsp`:

1. **Namespace:** `Hyphae` → `Fireside`
2. **Version:** `0.4.0-draft` → `0.5.0-draft`
3. **Navigation → Traversal:** Type and field rename
4. **ContentBlock union:** 11 variants → 7 core + `ExtensionBlock`
5. **Validation decorators:**
   - `@minValue(1)` / `@maxValue(6)` on `HeadingBlock.level`
   - `@minItems(1)` on `Graph.nodes`
   - `@minItems(1)` on `BranchPoint.options`
   - `@minLength(1)` on `Node.id`
   - `@minLength(1)` on `BranchOption.target`
6. **`$schema` field** on Graph for self-describing documents
7. **Versioning library:** Import `@typespec/versioning`, add version enum
8. **Operations interface:** Define `Next`, `Choose`, `Goto`, `Back` as TypeSpec operations
9. **ExtensionBlock model:**
   ```
   model ExtensionBlock {
     type: string; // must start with "x-"
     fallback?: ContentBlock; // required for conformance
     ...Record<unknown>; // arbitrary extension properties
   }
   ```
10. **Discriminator:** Add `@discriminator("type")` to ContentBlock union

### Post-revision: Recompile to JSON Schema 2020-12

Expected output: ~18-20 schema files (down from 21 due to removed block types, up slightly from ExtensionBlock + Traversal)

## Phase 6 — Vocabulary Purge

### Files to DELETE:

- `schemas/presentation.schema.json` — completely stale, old vocabulary
- `schemas/slide.schema.json` — completely stale, different Layout/Transition values, snake_case
- `specs/spec.md` — pre-Hyphae fossil using Journey/Waypoint/Marker/Crossroads

### Files to UPDATE (vocabulary sweep):

- All memory-bank files: Journey/Waypoint/Marker → Graph/Node terminology; Hyphae → Fireside
- `docs/astro.config.mjs`: title "Slideways" → "Fireside", description update
- All doc pages: grep sweep for old terms (Hyphae, Navigation, Journey, Waypoint, Marker, Crossroads, Slideways)

### Grep targets (must return 0 matches after purge):

```
Journey|Waypoint|Marker|Crossroads|Hyphae|"navigation"|Navigation
```

(Excluding historical references in changelogs/decision logs)

## Phase 7 — Documentation Rewrite

### Landing page (`index.md`):

- Fireside branding, warm voice
- Updated glossary with Traversal terminology
- New chapter table (6 + 3 appendices)
- Version: 0.5.0-draft, Protocol Name: Fireside

### Spec chapters (6 new files):

Fresh writing — not edited versions of old chapters. Each chapter:

- Uses RFC 2119 + RFC 8174 conformance language
- Derives data model content from TypeSpec output
- Defines traversal algorithms as numbered imperative steps
- References JSON Schema for validation rules

### Guides (3 files):

1. **"Your First Fireside Session"** — replaces getting-started.md, fixes wrong design token names, uses brand vocabulary
2. **"Branching Adventures"** — rewrite with Fireside voice
3. **"For Designers"** — NEW guide covering design tokens, theme creation, accessibility

### Schema documentation pages:

- Update all schema doc pages to reflect new TypeSpec output
- Rename graph.md, node.md etc. to match new model
- Add pages for Traversal, ExtensionBlock

## Phase 8 — Design Decisions Log

Create `docs/src/content/docs/decisions/` with ADR (Architecture Decision Record) format:

| ADR | Question                  | Answer                                                                                                                                                                                                               |
| --- | ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 001 | Why a graph?              | Graphs support cycles, branches, rejoins, shortcuts. Trees can't rejoin. Lists can't branch. Flat sequences are a degenerate case (already supported).                                                               |
| 002 | Why JSON?                 | Richest schema ecosystem (JSON Schema 2020-12), universal parsing, TypeSpec emits natively. YAML as authoring alternative.                                                                                           |
| 003 | Why TEA guarantees?       | Perfect match for content graph navigation. Prescribe guarantees (immutable graph, sequential ops, history stack, stateless render) without mandating architecture.                                                  |
| 004 | Why 7 core blocks?        | Minimal set covering 95% of use cases across all 8 user journeys. Extension model handles the rest.                                                                                                                  |
| 005 | Why `x-` prefix?          | Proven pattern from OpenAPI. Namespaces extensions clearly. Engines can ignore unknown extensions safely.                                                                                                            |
| 006 | Why not Twine/Ink format? | Twine is tool-specific (HTML-based). Ink is interpreter-dependent. Neither produces portable, schema-validated JSON. Fireside takes the best ideas (Ink's weave/gather ≈ `after` rejoin) in a format-first approach. |

---

## Execution Order

| Step | Action                                                                               | Depends On    |
| ---- | ------------------------------------------------------------------------------------ | ------------- |
| 1    | Delete stale files (schemas/\*.json, specs/spec.md)                                  | —             |
| 2    | Revise TypeSpec model (namespace, traversal, content blocks, validation, versioning) | —             |
| 3    | Recompile TypeSpec → JSON Schema 2020-12                                             | Step 2        |
| 4    | Write spec chapters §1-§6                                                            | Steps 2, 3    |
| 5    | Write appendices A-C                                                                 | Steps 2, 3    |
| 6    | Rewrite guides (3 files)                                                             | Steps 4, 5    |
| 7    | Update landing page + schema doc pages                                               | Steps 3, 4, 5 |
| 8    | Update support files (astro.config, memory-bank)                                     | Step 7        |
| 9    | Full vocabulary sweep (grep for old terms)                                           | Step 8        |
| 10   | Build validation (`npm run build` clean)                                             | Step 9        |

---

## Key Constraints

- **Wire format:** camelCase JSON (no change)
- **`@encodedName`:** Does not propagate to JSON Schema emitter — camelCase accepted as-is // let's not use this then, find alternatives.
- **Conformance language:** RFC 2119 + RFC 8174
- **TypeSpec version:** 1.9.0
- **Astro/Starlight:** 5.17 / 0.32
- **Schema dialect:** JSON Schema 2020-12

## Known Issues to Fix

1. No discriminator on ContentBlock union (anyOf without type discriminator)
2. HeadingBlock.level unconstrained (should be 1-6)
3. No minItems on Graph.nodes (empty graph is invalid)
4. No minItems on BranchPoint.options (branchless branch point is invalid)
5. §6/§7 breakpoint inconsistency
6. `after` override underspecified in §3 traversal algorithm
7. Getting-started guide invents wrong design token names (bg_primary, fg_primary)
8. Astro config title still says "Slideways"
9. Old vocabulary in memory-bank files (Journey, Waypoint, Marker)
10. Old vocabulary in specs/spec.md (Journey, Waypoint, Marker, Crossroads)

## Research Sources

- **DDD Starter Modelling Process:** https://github.com/ddd-crew/ddd-starter-modelling-process
- **GitHub Spec Kit:** https://github.github.com/spec-kit/
- **Existing formats analyzed:** Twine, Ink, ChoiceScript, SCORM/xAPI
- **Spec structures studied:** GraphQL, JSON Schema, OpenAPI, RFC patterns
- **TypeSpec docs:** Validation decorators, versioning library, operations/interfaces, augment decorators

## Additional feedback

- You are the architect, product owner, and champion of this protocol, empowered to use subagents to manage your work and break down the tasks into smaller steps. For example, you could create a subagent for each phase of the revision plan, or for specific tasks like updating the TypeSpec model or rewriting the guides. This will help you stay organized and focused as you work through the comprehensive revision process. Use "GPT-5.3 Codex" for code-related tasks and "GPT-5.2" for writing and documentation tasks, as appropriate. Try seeing what Gemini 3 Pro can do with the design language for the design system appendix, as it may be able to generate more creative and visually appealing content. The design system and language need to advocate for the user experience. However, I'd also like for you do do this manually to ensure it aligns with the brand identity and aesthetic direction we've defined for Fireside. Remember to keep the tone warm and inviting, and to use storytelling elements in the guides to make them engaging and accessible for users of all backgrounds. Use an appropriate model for a subagent that's solely focused on technical writing, such as "GPT-5.2", to ensure the content is clear, concise, and well-structured.
- Naming conventions
  - prefer kebab-case for property names in JSON Schema (e.g., `branch-point` instead of `branchPoint`) to align with common JSON API practices and improve readability
- AI Tooling
  - Based on the revision here, it's probably a good time to revise the copilot instructions.
  - There are certain flows, like compiling the TypeSpec to JSON Schema, that are very mechanical and could be fully automated with an AI skill file. For example, we know that once we have changes in the typespec directory, the next step is to run `npm run build` to recompile the JSON Schema. These in turn should trigger a updates to the documentation pages that reference the schema files. We could create a skill file that watches for changes in the typespec directory, runs the build command, and then automatically updates any doc pages that reference the changed schema files with the new version number and any relevant changes. This would save a lot of manual work and ensure consistency across the documentation.
- Slideways also needs to be rebranded to Fireside in the documentation, guides, and anywhere else it is mentioned. This includes updating the landing page, any references in the guides, and the glossary. We should also update the astro.config.mjs file to reflect the new title and description for the documentation site.
- I already renamed the directory and deleted the .git folder. Once you are done implementing this superceding revision plan, you can initialize a new git repository in the root of the project and make an initial commit with all the changes.
- The spec is the most important initial contribution from this repository. We will want this to be a rust monorepo that can then implement the spec in rust crates to cover core, cli, validation, rendering, and design system. The spec should be written in a way that is implementation-agnostic and focuses on the protocol itself, rather than any specific implementation details. This will allow for multiple implementations in different languages or frameworks in the future, while still adhering to the same core protocol defined in the spec.
