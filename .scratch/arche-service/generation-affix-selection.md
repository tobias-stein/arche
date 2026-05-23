---
title: Generation engine — affix selection
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement affix selection for the selected blueprint:

1. Separate the blueprint's affix pool into prefixes and suffixes (using `location` field on `blueprint_affixes`)
2. Apply `require` affixes from the request — these are pre-selected and counted against the roll
3. Exclude `block` affixes from the pool
4. Roll affix count per type (prefixes and suffixes separately):
   - Default distribution: uniform between blueprint's `min_prefixes`/`max_prefixes` (and same for suffixes)
   - If the request specifies `min_prefixes`/`max_prefixes`, those override the blueprint's values as hard requirements
5. Select N affixes from the remaining pool via weighted random **without replacement** (each affix appears at most once per generated thing)
6. If `require` + `block` make selection impossible (not enough affixes to satisfy min count), return an error

The selection uses the seeded RNG. Returns a list of selected `(affix, blueprint_affix)` pairs, ordered by `sort_order`.

## Acceptance criteria

- [ ] Prefixes and suffixes selected independently
- [ ] Weighted random without replacement: no duplicate affixes
- [ ] Required affixes: always included, counted against roll
- [ ] Blocked affixes: never selected
- [ ] Request min/max override blueprint min/max
- [ ] Insufficient affix pool to satisfy min count → error
- [ ] Deterministic with same seed + same input
- [ ] Affix count rolled with uniform distribution (default)
- [ ] Output ordered by sort_order
- [ ] Unit tests with deterministic seeds

## Blocked by

- arche-service/generation-engine-memory-load.md
- arche-service/affixes-crud.md
