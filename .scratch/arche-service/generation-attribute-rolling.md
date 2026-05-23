---
title: Generation engine — attribute resolution & rolling
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement attribute resolution and value rolling for the selected blueprint:

1. **Attribute resolution:** For each attribute key on the blueprint, if it has `$ref_id`, resolve from the global pool. Inline definitions override global ones (same key name). The effective attribute set is the union of inline + resolved globals, with inline winning on key collision.

2. **Value rolling** per value type:
   - `single`: return the stated float value
   - `enum`: select one value uniformly from the `values` array
   - `range`: roll a float according to the configured distribution:
     - Uniform: random value between min and max
     - Normal: Box-Muller transform, clamp to [min, max]
     - Exponential: sample from exponential distribution with given rate, clamp to [min, max]
   - `string`: return the stated string value (or null)
   - `boolean`: return `true` (payload is empty — presence means true)

All rolling uses a seeded RNG for reproducibility. The distribution config is validated at write time (covers all edge cases from validation rules).

Returns an ordered list of (key, rolled_value) pairs, in `attribute_order` sequence.

## Acceptance criteria

- [ ] `$ref_id` resolution: global definition loaded, inline overrides when key collision
- [ ] Single value rolling: returns the stated value
- [ ] Enum value rolling: selects uniformly, seeded determinism
- [ ] Range uniform: value between min and max, deterministic with seed
- [ ] Range normal: approximate normal distribution, clamped to [min, max]
- [ ] Range exponential: sampled from exponential, clamped to [min, max]
- [ ] String value: returns stated string
- [ ] Boolean: returns true
- [ ] Output ordered by attribute_order
- [ ] Unit tests with property-based testing (proptest) for distribution behavior
- [ ] Deterministic with same seed + same input

## Blocked by

- arche-service/generation-engine-memory-load.md
