---
title: arche-types — validation logic
status: ready-for-agent
---

## Parent

arche-types shared library (see `.scratch/arche-types/prd.md`)

## What to build

Implement validation functions in `arche-types` that enforce the consistency rules shared by both the service (write-time) and CLI (client-side). Functions return a `Result` or a list of validation errors.

Rules to enforce:
- `range.min` must be ≤ `range.max`
- `range.distribution.std_dev` must be > 0
- `range.distribution.rate` must be > 0
- `enum.values` must have at least 1 entry
- `single.value` must not be null
- An attribute cannot have both `$ref_id` and inline fields simultaneously
- `attribute_order` must contain exactly all keys in the `attributes` map, no duplicates
- Blueprint weight must be > 0
- Affix min/max counts: min ≤ max, min ≥ 0

Define a `ValidationError` struct with `path` and `message` fields. Validation functions aggregate errors rather than failing on the first one.

## Acceptance criteria

- [ ] `validate_attribute_payload(value_type, payload)` returns errors for all invalid states
- [ ] `validate_attribute_order(attribute_order, attributes)` returns errors for missing, extra, or duplicate keys
- [ ] `validate_blueprint(blueprint)` validates weight, affix counts, attributes, attribute_order
- [ ] All validation functions return a `Vec<ValidationError>` (empty = valid)
- [ ] Unit tests covering all validation rules (valid cases + every edge case)
- [ ] `cargo test` passes

## Blocked by

- arche-types/domain-structs-enums.md
