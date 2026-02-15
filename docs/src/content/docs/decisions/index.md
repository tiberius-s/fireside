---
title: 'Architecture Decision Records'
description: 'Index of Architecture Decision Records (ADRs) documenting the key design choices behind the Fireside Protocol.'
---

Architecture Decision Records (ADRs) capture the significant design choices made
during the development of the Fireside Protocol. Each record documents the
context that motivated a decision, the decision itself, and its consequences.

ADRs are immutable once accepted. If a decision is later reversed or superseded,
a new ADR is created referencing the original. This preserves the full history
of the protocol's evolution.

**Baseline:** These ADRs define decisions for protocol version `0.1.0`.

## Why ADRs?

The Fireside Protocol makes deliberate tradeoffs — choosing a graph over a tree,
JSON over a binary format, seven blocks over eleven. These choices aren't
obvious from the specification alone. ADRs ensure that future contributors and
implementors understand not just _what_ was decided, but _why_.

## Decision Log

- ADR-001 — [Why a graph?](/decisions/adr-001/) (Accepted)
- ADR-002 — [Why JSON?](/decisions/adr-002/) (Accepted)
- ADR-003 — [Why TEA guarantees?](/decisions/adr-003/) (Accepted)
- ADR-004 — [Why 7 core blocks?](/decisions/adr-004/) (Superseded)
- ADR-005 — [Why explicit extension type?](/decisions/adr-005/) (Superseded)
- ADR-006 — [Why not Twine/Ink?](/decisions/adr-006/) (Accepted)
