# Active Context

## Current Focus

Protocol and docs alignment for Fireside `0.1.0`.

## Recently Applied Direction

- Replaced `group` with `container` in protocol model and docs.
- Replaced `x-` prefix extension convention with explicit extension blocks:
  `kind: "extension"` + `type`.
- Standardized serialization guidance to `application/json`.
- Removed root `specs/` duplication by moving quick-reference docs into
  `docs/src/content/docs/reference/`.
- Enforced chapter ordering in docs sidebar: §1–§6 then appendices.

## Next Workstream

Align Rust reference implementation vocabulary with the protocol model where it
still uses legacy naming.
