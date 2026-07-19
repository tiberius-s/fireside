# Contract: `fireside art image` flags and behavior

## Command shape

```text
fireside art image <path> [--width N] [--charset <default|block|slight>] [--invert] [--no-normalize]
```

`path` and `--width` are unchanged from today. Three new flags, all
optional, all independent of each other (research.md §6):

| Flag              | Type / values                    | Default   | Effect                                                                 |
| ------------------ | --------------------------------- | --------- | -------------------------------------------------------------------------- |
| `--charset <name>` | `default` \| `block` \| `slight`  | `default` | Selects the character set used for shading. `default` matches today's unflagged output exactly. |
| `--invert`         | boolean flag                      | off       | Swaps light/dark shading, independent of charset choice.                |
| `--no-normalize`   | boolean flag                      | off       | Disables the automatic contrast stretch (below); output matches this feature's pre-existing behavior exactly. |

## Automatic contrast stretch (default-on behavior)

Unless `--no-normalize` is given, before converting to ASCII the tool:

1. Computes the 2nd and 98th percentile luma values (`lo`, `hi`) across the
   source image's pixels.
2. If `hi > lo`, linearly remaps every color channel of every pixel so that
   `lo` maps to `0` and `hi` maps to `255` (values outside `[lo, hi]` clamp
   to the nearest end) — a standard percentile-based levels stretch.
3. If `hi <= lo` (a solid-fill or otherwise zero-range image), the image is
   left unchanged — no division by zero, no distortion.
4. The (possibly stretched) image is handed to the existing `rascii_art`
   conversion with the requested `width`, `charset`, and `invert` settings.

This step runs regardless of `--charset`/`--invert` choice — it operates on
pixel values before charset mapping happens (research.md §2).

## Low-range warning

Using the *same* `(lo, hi)` values computed above (whether or not the
stretch was actually applied), if `hi - lo < 102` (roughly 40% of the full
0–255 range), the tool prints one note to stderr naming the approximate
percentage of the range used and suggesting `--invert` or a higher-contrast
source image. This note:

- Fires regardless of `--no-normalize` (it describes the *source* image,
  not the output).
- Never appears when the pre-stretch range is at or above the threshold.
- Never blocks or alters stdout — the full converted art still prints.

## Non-goals

- No flag lets a user tune the 2%/98% percentile cutoffs or the 40% warning
  threshold themselves — only the stretch as a whole can be disabled
  (`--no-normalize`).
- `fireside art text` and `new --banner`'s text-banner generation are
  entirely unaffected — none of the flags or behavior above apply to that
  path (FR-008).
- No change to `--width`'s existing meaning or default.

## Exit codes

Unchanged: `0` on success; `1` if `path` doesn't exist or isn't a readable
image (existing behavior, still reported with a clear message, never a
panic). None of the new flags introduce a new failure mode — an unknown
`--charset` value is rejected by `clap` itself with its standard usage
error, same as an unknown `--template` value on `fireside new` today.
