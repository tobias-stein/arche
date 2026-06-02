---
title: Support gte/lte constraints on Single value type attributes
status: ready-for-agent
---

## Problem

Arche's `Single` attribute payload currently only matches against `eq` constraints. To support a fixed-level content model (where each blueprint has a single integer level instead of a range), the generation engine must also accept `gte`/`lte` constraints on `Single` values.

## Design

See `.scratch/dungeon-crawler/stat-reference-curve.md` for the stat curve design context. See `docs/adr/` for the ADR covering the level model change.

## Specification

### Constraint matching

In `payload_matches_constraint` (`crates/arche-service/src/generation/blueprint_selection.rs:90-94`), change the `Single` arm from:

```rust
AttributePayload::Single { value, .. } => {
    config.eq.as_ref().is_some_and(|v| {
        v.as_f64().is_some_and(|v2| (v2 - value).abs() < f64::EPSILON)
    })
}
```

to:

```rust
AttributePayload::Single { value, .. } => {
    let eq_ok = config.eq.as_ref().is_none_or(|v| {
        v.as_f64().is_some_and(|v2| (v2 - value).abs() < f64::EPSILON)
    });
    let gte_ok = config.gte.map_or(true, |gte| *value >= gte);
    let lte_ok = config.lte.map_or(true, |lte| *value <= lte);
    eq_ok && gte_ok && lte_ok
}
```

This enables sending `{ level: { gte: 3, lte: 7 } }` against blueprints with `{ "value_type": "single", "value": 5 }`.

### Tests

- `test_single_gte_lte_in_range` — gte=3, lte=7, value=5 → true
- `test_single_gte_out_of_range` — gte=10, value=5 → false
- `test_single_lte_out_of_range` — lte=3, value=7 → false
- `test_single_eq_only_backward_compat` — eq=42, value=42 → true
- `test_single_gte_lte_with_eq_combined` — gte=1, lte=10, eq=5, value=5 → true

### Files to change

- `crates/arche-service/src/generation/blueprint_selection.rs` — constraint matching logic + tests

### Backward compatibility

No breaking changes. Existing `eq`-only usage continues to work. `gte`/`lte` are additive.

### Verification

```
cargo test -p arche-service -- blueprint_selection::tests
```