---
title: 'Extension Authoring'
description: 'Define typed extension blocks with stable fallbacks and safe payload handling.'
---

Fireside extensions let you add domain-specific blocks without breaking engines
that only support the core content model.

## Wire Format

Every extension block uses the standard `ContentBlock` discriminator with
`"kind": "extension"`.

```json
{
  "kind": "extension",
  "type": "dev.fireside.mermaid",
  "payload": {
    "diagram": "graph TD; A-->B"
  },
  "fallback": {
    "kind": "code",
    "language": "mermaid",
    "source": "graph TD; A-->B"
  }
}
```

## Required and Optional Fields

| Field      | Required    | Meaning                                          |
| ---------- | ----------- | ------------------------------------------------ |
| `kind`     | yes         | Must be `extension`                              |
| `type`     | yes         | Extension identifier                             |
| `payload`  | yes         | Extension-specific data object/value             |
| `fallback` | recommended | Renderable core block for non-supporting engines |

## `type` Naming Convention

Use reverse-domain naming for global uniqueness:

- `dev.fireside.mermaid`
- `com.example.timeline`
- `io.acme.metric-card`

Avoid short generic names like `chart` or `widget`.

## Fallback Contract

`fallback` is the compatibility path when an engine does not implement your
extension type.

Authoring guidance:

- Always provide a fallback for portable documents.
- Keep fallback semantically equivalent, even if visually simpler.
- Prefer core block types (`text`, `code`, `container`) so every engine can
  display meaningful output.

## Payload Shape

`payload` is intentionally flexible (`serde_json::Value` at runtime). Treat it
as a versioned contract owned by the extension author.

Suggested payload pattern:

```json
{
  "version": 1,
  "data": {
    "...": "..."
  }
}
```

This keeps forward evolution explicit when your extension grows.

## Graph-Level Extension Declarations

At graph root, use `extensions` to declare extension dependencies:

```json
{
  "fireside-version": "v0-1-0",
  "extensions": [
    { "type": "dev.fireside.mermaid", "required": false },
    { "type": "com.example.timeline", "required": true }
  ]
}
```

Declaration guidance:

- `required: true` means a renderer should fail clearly if unsupported.
- `required: false` means fallback rendering is acceptable.

## Engine Responsibilities

Engines should:

1. inspect `kind == extension`
2. dispatch by `type`
3. validate `payload` against extension-specific rules
4. render extension output when supported
5. otherwise render `fallback` when present

## Safety Guidance

Extension payloads are data, not executable code.

- Do not evaluate payload strings as code.
- Do not execute shell commands from payload values.
- Validate shape and bounds before rendering.
