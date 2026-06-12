# fireside-core

The Fireside 0.1.0 protocol data model, exactly: parsing, serialization, and
small read-time helpers (traversal accessors, view-mode/transition default
resolution). Mirrors the JSON schemas generated from `protocol/main.tsp` —
nothing more.

No I/O, no validation logic, no rendering. Dependencies: `serde`,
`serde_json`, `thiserror`.

```rust
let graph = fireside_core::Graph::from_json(text)?;
```

Unknown JSON fields are ignored on read (the schema layer owns strictness);
absent optional fields stay absent on write, so load → save round-trips are
faithful.
