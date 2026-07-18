# Contract: the `ascii-art` ContentBlock kind

## Wire shape

```json
{
  "kind": "ascii-art",
  "art": "  ___\n /   \\\n \\___/",
  "alt": "a simple circle",
  "reveal": 1
}
```

- `kind`: literal `"ascii-art"`. Required, discriminator.
- `art`: the pre-rendered, multi-line art content, as plain text.
  Required. MUST NOT contain ANSI color/formatting escape codes (this
  project's styling flows through the theme system, never embedded codes
  — constitution Principle IV).
- `alt`: optional plain-language description, for anyone who can't see
  the rendered art. Not displayed on screen; carried for accessibility/
  future export tooling, same purpose `ImageBlock.alt` already serves.
- `reveal`: optional, standard `Revealable` field, identical semantics to
  every other block kind (see `specs/006-incremental-reveal/contracts/reveal-field.md`
  — unchanged by this feature, reused as-is).

## Rendering contract

- Rendered centered within the available content width, sized to the
  art's own widest line — never stretched to the full content width. This
  is the same visual treatment the pre-existing language-less/`"text"`/
  `"ascii"` code-block path already gives ASCII art (spec 005); this block
  kind exists so that treatment has an unambiguous, purpose-built home
  instead of overloading `CodeBlock`.
- Reveals as one indivisible unit — an `ascii-art` block's lines all
  appear together on the presenter action that reaches its `reveal` step,
  never partially. (This is not a new mechanism — it already follows from
  the general block-level `is_revealed()` filter in `blocks.rs`, which
  every block kind uses; there is no per-line reveal concept anywhere in
  the protocol.)
- Participates in `reveal-masked-by-container` exactly like every other
  block kind — no `ascii-art`-specific validator interaction beyond the
  two new rules below.

## Validation contract

- `ascii-art-too-wide` (WARNING): the block's widest line (by display
  width) exceeds 76 columns.
- `ascii-art-empty` (WARNING): the block's `art` is empty or
  whitespace-only.
- Neither rule blocks presenting (WARNING, not ERROR) — consistent with
  every other content-quality check in this validator
  (`malformed-link-url`, `reveal-masked-by-container`).

## Compatibility

Unlike `reveal` (0.1.2, an additive optional field an old engine safely
ignores), `ascii-art` is a **new enum member** in a closed, tagged union.
An engine built before protocol 0.1.3 — including this project's own
reference implementation before this feature ships — has no branch for
`"kind": "ascii-art"` and MUST reject the whole document rather than
silently drop or misrender the block. In the reference implementation
this is not new code to write: it already falls out of
`fireside-core::model::ContentBlock`'s existing `#[serde(tag = "kind")]`
closed-enum design, which produces (verified directly, `research.md` §2):

```text
not a valid Fireside document: unknown variant `ascii-art`,
expected one of `heading`, `text`, `code`, `list`, `image`,
`divider`, `container` at line 1 column 50
```

surfaced through the same whole-document parse-error path
(`fireside-cli`'s `report::parse_report`) every other malformed document
already uses. A conforming third-party engine SHOULD do the same: reject
the document with a clear message rather than attempt a partial or
silent render. This is a deliberate, named compatibility break — the
first one this protocol has shipped — and is recorded in ADR-012.
