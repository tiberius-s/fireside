# Contract: `art text` width guard

## Behavior

`fireside art text <phrase>` measures its rendered banner's widest line the
same way `new.rs::add_title_banner` already does
(`art.lines().map(str::len).max().unwrap_or(0)`) and compares it against
`art::DEFAULT_ART_WIDTH` (76 — the same constant `ascii-art-too-wide`
validates against and `new --banner` already compares against).

- Widest line `<= 76`: no change from today. stdout gets the banner, stderr
  gets nothing.
- Widest line `> 76`: stdout still gets the full, untruncated banner
  (unchanged — this command's job is to hand back pasteable art, not to
  reject it). stderr additionally gets one note naming the measured width,
  after the stdout write.

## Non-goals

- No new flag, no truncation, no auto-shrink. This is a heads-up, not a
  constraint — matches the deferred-item note in the source UX plan ("`art
  text` font options / auto-shrink — only if users hit the width guard
  often").
- `fireside art image` is unaffected — this contract is text-banner-only,
  matching the feature description's scope.
- `new --banner`'s existing skip-note behavior (silently omitting a banner
  that doesn't fit rather than warning about it) is unchanged; this is a
  separate, additive note on the standalone `art text` verb only, which has
  no deck to skip a block from.

## Exit code

Unchanged — `art text` still exits 0 whether or not the note fires; a wide
banner is advisory, not an error, consistent with `ascii-art-too-wide`
being a `Warning` rather than an `Error`.
