---
title: "Benchmark: Synthetic data generator (all types/distributions)"
status: completed
---

## Parent

PRD: `.scratch/benchmark-suite/prd.md`

## What to build

Replace the hardcoded range-only attribute seeding with a synthetic data generator that:

1. For each attribute of a blueprint, randomly picks one of the 5 value types (single, enum, range, string, boolean)
2. For range-type attributes, randomly picks one of 3 distributions (uniform, normal, exponential) with appropriate parameters
3. For enum-type attributes, generates 3-6 random enum option strings
4. Creates sensible defaults for each type (e.g. range: min=0, max=100; single: value=50.0; boolean: true/false)
5. All randomness is **deterministic** — seeded by a hash of the scenario descriptor tuple
6. Blueprint names remain `Blueprint-{n:04d}`, affix names `Affix-{n:04d}-{prefix|suffix}`
7. Seeds `global_meta_attributes` for a small number of `$ref_id`-based attributes (maybe 1 in 5 attributes uses a global ref)

## Acceptance criteria

- [ ] Running the seeder twice with the same scenario produces identical blueprints/affixes
- [ ] Running with different scenarios produces different data
- [ ] All 5 attribute types appear across the generated data
- [ ] All 3 distributions appear among range-type attributes
- [ ] 1 in 5 attributes uses a `$ref_id` to a global meta attribute
- [ ] `Pytest` tests verify determinism and type coverage

## Blocked by

- `bench-03-data-seeding` (replaces its hardcoded logic)
