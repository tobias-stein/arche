---
title: Pre-resolve blueprint attributes at cache-load time
status: completed
---

## What to build

Every generate request deserializes blueprint attributes from `serde_json::Value` into `BlueprintAttribute` enums using `serde_json::from_value()`. This happens in `blueprint_selection.rs:71` (constraint matching) and `rolling.rs:38` (attribute rolling). The `BlueprintAttribute` enum is `#[serde(untagged)]`, which means serde tries each variant in order — this is slower than a tagged representation.

This deserialization parses purely static data (the blueprint definition never changes between cache reloads). Move it to cache-load time so it happens once instead of once per request.

**This does NOT affect randomness.** The resolved `AttributePayload` only defines *what* to roll (e.g., min=10, max=20, uniform distribution). The actual random value rolling still happens per-request in `roll_range()` and `roll_attribute()`.

**Design:**

- In `cache.rs`, store pre-resolved attributes per blueprint. Add a new map like `blueprint_resolved_attributes: HashMap<Uuid, HashMap<String, AttributePayload>>` to the cache.
- During `build_client_caches()` (or a new post-load step), iterate all blueprints and deserialize their attributes + resolve `$ref_id` references once, storing the result.
- Store a `HashMap<Uuid, HashMap<String, AttributePayload>>` in `ClientCache` keyed by blueprint ID.
- In `resolve_attributes()` in `rolling.rs`, use the pre-resolved map instead of calling `serde_json::from_value()`.
- In `resolve_payload()` in `blueprint_selection.rs`, use the pre-resolved map instead of calling `serde_json::from_value()`.
- `AffixAttribute` resolution can be handled similarly — pre-resolve the affix attribute payload at cache-load time.

**Acceptance criteria:**

- [ ] Pre-resolved attribute map is populated at cache-load time
- [ ] `resolve_attributes` uses pre-resolved data, no per-request serde_json calls
- [ ] `resolve_payload` (constraint matching) uses pre-resolved data
- [ ] Affix attributes are also pre-resolved where possible
- [ ] All generation tests still pass (same deterministic output)
- [ ] Benchmark shows improvement, especially for attr=10 and attr=25 scenarios

## Blocked by

None — can start immediately
