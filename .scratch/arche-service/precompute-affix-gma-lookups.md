---
title: Pre-compute affix_by_id and gma_by_id lookup maps in cache
status: ready-for-agent
---

## What to build

Currently the generation engine does two inefficient lookups on every request:

1. `affix_selection.rs:43-44` — rebuilds `HashMap<Uuid, &Arc<Affix>>` from the full affix list on every single generate request
2. `rolling.rs:46-48` and `blueprint_selection.rs:75` — does O(n) linear search `gmas.iter().find(|g| g.id == ref_id)` to resolve `$ref_id` references

Both of these should be pre-computed at cache-load time and stored in `ClientCache`.

**Design:**

- Add `affix_by_id: HashMap<Uuid, Arc<Affix>>` and `gma_by_id: HashMap<Uuid, Arc<GlobalMetaAttribute>>` fields to the `ClientCache` struct in `cache.rs`.
- Build these maps in `build_client_caches()` alongside the existing Vecs.
- In `select_affixes()`, accept a `&HashMap<Uuid, &Arc<Affix>>` parameter instead of rebuilding it.
- In `resolve_payload()` and `resolve_attributes()`, use `gma_by_id.get(&ref_id)` instead of `gmas.iter().find(...)`.
- In `resolve_and_roll_affix_attribute()`, use the same HashMap lookup.

**Acceptance criteria:**

- [ ] `ClientCache` has `affix_by_id` and `gma_by_id` HashMap fields
- [ ] Maps are populated at cache load time (in `build_client_caches`)
- [ ] `select_affixes` no longer rebuilds the HashMap every request
- [ ] `$ref_id` resolution uses O(1) HashMap lookup instead of O(n) linear search
- [ ] All generation tests still pass
- [ ] Benchmark shows measurable improvement in attribute-heavy scenarios (attr=10, attr=25)

## Blocked by

None — can start immediately
