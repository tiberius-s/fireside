# Quickstart: validating ASCII Art Image Quality

Prerequisites: built binary (`cargo build -p fireside-cli`), a low-contrast
test image and a high-contrast test image (any two local photos, or the
project's `.github/demo-art.png` before/after replacement — see
`research.md` §7). Each scenario checks against `contracts/art-image-cli.md`.

## 1. Default output is recognizable on a low-contrast photo (US1)

```sh
fireside art image /path/to/low-contrast-photo.jpg
```

Expect: the printed ASCII art visibly uses a wider range of shading
characters than a direct/unstretched conversion would, and a person who
hasn't seen the source can identify the subject. Compare against the
pre-feature behavior via the opt-out:

```sh
fireside art image /path/to/low-contrast-photo.jpg --no-normalize
```

Expect: this reproduces the old (muddy/undecipherable) output exactly —
confirms the opt-out is a true escape hatch, not an approximation.

## 2. No visible regression on an already-high-contrast photo (US1)

```sh
fireside art image /path/to/high-contrast-photo.jpg
```

Expect: output looks effectively the same with and without `--no-normalize`
— the stretch is a no-op (or negligible) when the source already spans the
full range.

## 3. Charset and invert flags (US2)

```sh
fireside art image /path/to/photo.jpg --charset block
fireside art image /path/to/photo.jpg --charset slight
fireside art image /path/to/photo.jpg --invert
```

Expect: each command produces valid, differently-shaded output — `block`
uses block-shading glyphs (`░▒▓█`), `slight` a sparser character set,
`--invert` swaps light/dark relative to a non-inverted run of the same
image. Confirm the default is unchanged:

```sh
fireside art image /path/to/photo.jpg
fireside art image /path/to/photo.jpg --charset default
```

Expect: identical output between these two invocations.

## 4. Low-range warning (US3)

```sh
fireside art image /path/to/flat-featureless-photo.jpg
```

Expect: stderr shows a note naming the image's brightness range as unusually
narrow, suggesting `--invert` or a higher-contrast source; stdout still
shows the full converted output. Confirm silence on a normal photo:

```sh
fireside art image /path/to/photo.jpg
```

Expect: no such stderr note.

## 5. Documentation example (US4)

Follow `reference/cli.md`'s `fireside art image` section exactly (its
updated example image and command). Expect: the output you get matches
what the page shows, and clearly resembles the depicted subject — not the
previous night-photo example's muddy result.

## Full verification

After implementing, run the project's standard gates before calling this
feature done:

```sh
cargo +1.88 build -p fireside-cli   # confirm the new `image` dependency under MSRV
cargo test --workspace
cargo clippy --workspace --all-targets
scripts/verify.sh
scripts/demos.sh                     # regenerate art-image.gif against the new demo image
graphify update .
```

No TUI-visible surface is touched by this feature (it's a CLI-only, stdout/
stderr change), so constitution Principle VII's tmux-smoke requirement does
not apply — but do visually eyeball the regenerated `art-image.gif` and the
`reference/cli.md` before/after presentation, since "is it recognizable" is
the entire point of this feature and no automated test can fully substitute
for a human looking at it.
