# Feature Specification: ASCII Art Image Quality

**Feature Branch**: `011-art-image-quality`

**Created**: 2026-07-18

**Status**: Draft

**Input**: User description: "art image output quality — from .claude/plans/2026-07-18-ux-polish-plan.md Wave 3 (spec 011). Root cause verified experimentally: fireside-cli's `render_image_ascii` (crates/fireside-cli/src/art.rs:46) passes rascii_art defaults with only linear grayscale→charset mapping, so low-contrast source photos (e.g. .github/demo-art.png, using only ~29% of the luminance range) land almost every cell in the two darkest glyphs and produce undecipherable ASCII art. Scope: (1) auto contrast-stretch by default via a 2%/98% percentile clip on the loaded image before handing it to rascii_art::render_image_to, with an escape hatch flag `--no-normalize`; (2) surface rascii_art 0.4.5's existing `--charset <default|block|slight>` and `--invert` flags on the CLI; (3) a low-range warning printed to stderr when the pre-stretch 2-98% span is under ~40% of the full brightness range, suggesting --invert or a higher-contrast image; (4) replace the demo image with a high-contrast, simple-silhouette CC0 subject and re-record art-image.gif via scripts/demos.sh, keeping the input-photo-next-to-output presentation pattern in reference/cli.md; (5) update reference/cli.md's flag table and guides/authoring-markdown.md's pointer — `new --banner` text-banner path is unaffected since it doesn't normalize. Acceptance: converting the demo image with default flags yields recognizable output; goldens covering the contrast-stretch math; e2e tests for the new flags and the warning path."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Get a recognizable ASCII conversion from an ordinary photo (Priority: P1)

An author runs `fireside art image` against a normal photo they had lying
around — not one specially edited for high contrast — and expects the
printed ASCII art to actually resemble the subject. Today, a photo that
uses only a narrow slice of the brightness range (a common trait of ordinary
indoor or evening photos) converts to a wall of near-identical dark glyphs,
because the darkest and lightest pixels in the source are mapped straight to
character shades without first spreading them across the available range.
The fix should apply automatically, with no flags to learn, for the common
case.

**Why this priority**: This is the actual complaint driving the whole
feature — "undecipherable output" — and it's the one change every other
story in this feature builds on or coexists with.

**Independent Test**: Convert a known low-contrast source image with default
flags and confirm the output is a recognizable rendering of the subject (a
human reviewer, or a golden-output comparison, can identify the silhouette),
where the same image without the fix was not recognizable.

**Acceptance Scenarios**:

1. **Given** a source image whose brightness values span only a narrow slice
   of the full range, **When** it is converted with `fireside art image` and
   no extra flags, **Then** the printed output uses a visibly wider spread of
   character shades than a direct, unstretched conversion would, and the
   subject is recognizable.
2. **Given** a source image that already uses the full brightness range,
   **When** it is converted with default flags, **Then** the output looks
   effectively the same as before this feature (no visible degradation from
   applying a stretch to an image that didn't need one).
3. **Given** a user who wants the old, unadjusted behavior for some reason,
   **When** they pass an explicit opt-out flag, **Then** the output matches
   what `fireside art image` produced before this feature shipped.

---

### User Story 2 - Choose a different look for the conversion (Priority: P2)

An author isn't happy with the default character shading, or the subject is
naturally light-on-dark (or vice versa) such that even a well-stretched
conversion still reads oddly. They want to pick a different character set or
flip light/dark without leaving the command line.

**Why this priority**: A direct lever for the cases the automatic contrast
fix doesn't fully resolve, and a self-contained flag surface change with no
dependency on the stretch math in User Story 1.

**Independent Test**: Convert the same image three times with different
charset choices (and once with the invert flag) and confirm each produces
visibly different, valid output using only flags — no code changes needed to
verify.

**Acceptance Scenarios**:

1. **Given** a source image, **When** converted with each of the supported
   character-set choices, **Then** each produces a valid ASCII rendering
   using a visibly different set of characters for shading.
2. **Given** a source image, **When** converted with the invert option,
   **Then** light and dark areas of the output are swapped relative to a
   non-inverted conversion of the same image.
3. **Given** no character-set flag is given, **When** the image is converted,
   **Then** the result matches today's default character set (no behavior
   change for users who don't ask for something different).

---

### User Story 3 - Find out why a conversion still looks muddy (Priority: P3)

An author converts an image that's so low-contrast that even the automatic
stretch can't produce a clearly legible result (for example, a mostly flat,
featureless photo). Instead of silently producing poor output and leaving
them to guess why, the tool should tell them, and suggest what to try next.

**Why this priority**: A diagnostic aid on top of the two behavior changes
above — valuable, but strictly lower priority than actually improving output
(P1) or giving users manual control (P2), since it doesn't change any pixel
of the output itself.

**Independent Test**: Convert a deliberately flat/featureless test image and
confirm a warning appears on stderr while the image still converts and
prints output on stdout (the warning never blocks the command).

**Acceptance Scenarios**:

1. **Given** a source image whose brightness values are unusually
   concentrated in a narrow band even before any adjustment, **When** it is
   converted, **Then** a note appears on stderr suggesting the invert flag or
   a higher-contrast source image, and stdout still contains the full
   converted output.
2. **Given** a source image with an ordinary, reasonably spread brightness
   range, **When** it is converted, **Then** no such warning appears.

---

### User Story 4 - Trust the documentation's example (Priority: P4)

Someone reading the CLI reference or the authoring guide sees a real
before/after example of `fireside art image` and expects that running the
command against the shown input photo produces output resembling the shown
GIF. Today's example photo is exactly the kind of low-contrast image this
feature fixes, and even after the fix it's a poor showcase (a busy, low-
detail night scene) — a confident, simple-subject example serves new users
better regardless of the underlying fix.

**Why this priority**: Documentation-only; it doesn't change any runtime
behavior and depends on the other stories being done first so the example
reflects the improved output.

**Independent Test**: Follow the documented example (source image plus
command) and confirm the actual output resembles both the documented output
and the depicted subject.

**Acceptance Scenarios**:

1. **Given** the documented example image and command, **When** a reader
   runs it themselves, **Then** the output they get matches the documented
   before/after presentation and clearly resembles a simple, recognizable
   subject.
2. **Given** the CLI reference's flag table, **When** a reader looks up the
   image conversion command, **Then** every flag this feature adds is listed
   with its effect and default.

---

### Edge Cases

- An image that is already fully saturated black-and-white (only pure black
  and pure white pixels) — a percentile-based stretch must not divide by
  zero or otherwise fail; output should be unchanged from a direct
  conversion.
- A charset or invert choice is combined with the opt-out (no-normalize)
  flag — the two are independent controls and must compose (opting out of
  contrast-stretching doesn't disable charset/invert choice).
- A source path that doesn't exist or isn't a readable image — unaffected by
  this feature; the existing clear error behavior continues to apply.
- A source image with only one color (a solid fill) — brightness range is
  zero-width; the low-range warning must fire without crashing, and the
  stretch must leave such an image unchanged rather than producing
  undefined output.
- The text-banner path (`fireside art text`, `new --banner`) — entirely out
  of scope; this feature only touches image conversion.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `fireside art image` MUST, by default, adjust the source
  image's brightness values so they span the output's full available shading
  range before converting to characters, rather than mapping the source's
  raw (potentially narrow) brightness range directly.
- **FR-002**: The adjustment in FR-001 MUST be based on the image's actual
  brightness distribution (ignoring a small fraction of extreme outlier
  pixels at each end) rather than its absolute darkest and lightest single
  pixels, so a few stray very-dark or very-light pixels don't defeat the
  adjustment.
- **FR-003**: A user MUST be able to opt out of the FR-001 adjustment via an
  explicit flag, in which case the output matches the tool's pre-feature
  (unadjusted) behavior exactly.
- **FR-004**: A user MUST be able to choose among at least three distinct
  character sets for shading the output, with the tool's current default
  behavior preserved as one of the choices and used when none is specified.
- **FR-005**: A user MUST be able to invert light/dark shading in the
  output via a flag, independent of the character-set choice.
- **FR-006**: When a source image's brightness distribution (before any
  adjustment) is concentrated in less than roughly 40% of the full possible
  range, the tool MUST print a note to stderr identifying the condition and
  suggesting the invert flag or a higher-contrast source image, without
  altering or blocking the normal stdout output.
- **FR-007**: The low-range note in FR-006 MUST NOT appear for images whose
  brightness distribution already spans a wider range than that threshold.
- **FR-008**: None of the flags or adjustments introduced by this feature
  MUST affect the text-banner generation path (`fireside art text`,
  `new --banner`) — only image-file conversion is in scope.
- **FR-009**: The published CLI reference MUST document every new flag this
  feature adds (its purpose and default), and the example image/GIF shown
  alongside the image-conversion command MUST be a high-contrast, easily
  recognizable subject that demonstrates a clearly legible result.

### Key Entities

- **Brightness range**: The spread between an image's darkest and lightest
  meaningfully-represented pixel values, used both to decide whether to warn
  (FR-006) and as the basis for the stretch adjustment (FR-001).
- **Character set**: A named collection of characters used to represent
  shading steps from darkest to lightest in the printed output; the tool
  offers a fixed, small choice of these.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Converting the project's own documented example image with
  default flags produces output in which the subject is identifiable by a
  person who has not seen the source image, where it previously was not.
- **SC-002**: A user converting a normal, unedited photo with no flags gets
  a clearly improved (wider tonal spread) result compared to today's
  behavior, with no additional steps required.
- **SC-003**: A user unhappy with the default look can reach a different,
  valid rendering (different charset and/or inverted) using flags alone, no
  external tool needed.
- **SC-004**: A user converting an unusually flat image is told, at the time
  of conversion, that the result may be hard to read and what to try
  instead — rather than silently getting poor output with no explanation.
- **SC-005**: Every flag this feature adds is documented in the CLI
  reference with its default and effect.

## Assumptions

- "Full available shading range" means the complete set of character
  shades the chosen character set offers, from its darkest-representing
  character to its lightest — the stretch adjustment's job is to map the
  source image's actual (narrower) brightness spread onto that full set
  rather than leaving most of it unused.
- The percentile cutoffs used to ignore outlier pixels (2% at each end, per
  the feature description) and the low-range warning threshold (roughly 40%
  of full range) are reasonable defaults, not requirements a user can
  currently tune via flags — only the contrast-stretch as a whole can be
  disabled (FR-003).
- The default character set remains exactly what `fireside art image`
  already produces today; this feature only adds alternatives, it doesn't
  change the unflagged default look's character choice.
- Replacing the documented example image (FR-009) is a documentation and
  asset change only — it does not imply any change to conversion behavior
  beyond what FR-001–FR-007 already specify.
- This feature is scoped to the `fireside art image` conversion path only;
  it has no effect on deck validation rules, the wire protocol, or any
  other `fireside art`/`new --banner` text-based path.
