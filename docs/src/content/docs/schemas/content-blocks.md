---
title: 'Content Blocks'
description: 'Schema reference for core content blocks and the typed extension block model.'
---

`ContentBlock` is a discriminated union using the `kind` field.

## Core Kinds

| Kind        | Type             |
| ----------- | ---------------- |
| `heading`   | `HeadingBlock`   |
| `text`      | `TextBlock`      |
| `code`      | `CodeBlock`      |
| `list`      | `ListBlock`      |
| `image`     | `ImageBlock`     |
| `divider`   | `DividerBlock`   |
| `container` | `ContainerBlock` |

## ContainerBlock

| Property   | Type             | Required |
| ---------- | ---------------- | -------- |
| `kind`     | `"container"`    | Yes      |
| `children` | `ContentBlock[]` | Yes      |
| `layout`   | `string?`        | No       |

## ExtensionBlock

| Property         | Type            | Required |
| ---------------- | --------------- | -------- |
| `kind`           | `"extension"`   | Yes      |
| `type`           | `string`        | Yes      |
| `fallback`       | `ContentBlock?` | No       |
| `publisher`      | `string?`       | No       |
| `schema-version` | `string?`       | No       |
| `...`            | `unknown`       | No       |

Unsupported extension types SHOULD render fallback content when present.
