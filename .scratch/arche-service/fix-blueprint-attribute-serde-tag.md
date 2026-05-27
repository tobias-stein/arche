---
title: Fix BlueprintAttribute serde tag mismatch
status: ready-for-agent
---

## What to build

The `AttributePayload` enum in `crates/arche-types/src/attribute.rs` uses `#[serde(tag = "value_type")]` (snake_case), but the frontend sends `valueType` (camelCase) — consistent with the rest of the camelCase API. This causes two errors:

1. `"missing field 'value_type'"` — when the frontend sends an inline attribute with `"valueType": "single"`, serde can't find the expected `"value_type"` key
2. `"data did not match any variant of untagged enum BlueprintAttribute"` — after Inline fails (no `value_type`), Ref also fails (no `$ref_id`)

Change the serde tag on `AttributePayload` from `"value_type"` to `"valueType"` to match the frontend's camelCase convention.

Also verify the same pattern isn't broken for `CreateGlobalMetaAttributeRequest` which also uses `#[serde(flatten)]` with `AttributePayload`.

## Acceptance criteria

- [ ] `AttributePayload` serde tag changed from `"value_type"` to `"valueType"`
- [ ] All `AttributePayload` round-trip serialization tests pass
- [ ] Creating a blueprint with inline attributes via the API works end-to-end
- [ ] Creating a blueprint with `$ref_id` attributes via the API still works

## Blocked by

None — can start immediately.
