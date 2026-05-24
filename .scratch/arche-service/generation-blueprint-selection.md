---
title: Generation engine — blueprint selection
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement the blueprint selection phase of the generation engine:

1. Filter blueprints by archetype (if specified in the request)
2. Filter by constraint overlap semantics (ADR 0002):
   - For `gte`/`lte` constraints, check if the blueprint's range overlaps with the constraint range
   - For `in` constraints, check if the blueprint's enum values overlap with the constraint values
   - For `contains` constraints, check if the blueprint's string value contains the substring
   - For boolean constraints, check exact match
   - Multiple constraints are ANDed together
   - Only blueprints that explicitly define the constrained attribute (inline or `$ref_id`) can match
3. Select one blueprint from the matching set via weighted random (using the blueprint's `weight` field and the request's seed)

The selection is deterministic given the same seed + same data set. Takes the cache's `ClientCache`, the generate request, and a random number generator (seeded).

Returns the selected `Arc<Blueprint>` or an error if no blueprints match.

## Acceptance criteria

- [ ] Archetype filter: only matching blueprints considered
- [ ] Constraint matching: range overlap semantics work correctly (gte, lte, gte+lte combined)
- [ ] Enum constraint matching (`in`) works correctly
- [ ] String constraint matching (`contains`) works correctly
- [ ] Boolean constraint matching works correctly
- [ ] Multiple constraints: all must match (AND logic)
- [ ] Blueprint without the constrained attribute does NOT match (even if another blueprint has it)
- [ ] Weighted random selection: with fixed seed, same blueprint selected (deterministic)
- [ ] Weighted random: higher-weight blueprints selected more often over many runs
- [ ] No matching blueprints → error
- [ ] Unit tests with deterministic seeds verify exact selected blueprint

## Blocked by

- arche-service/generation-engine-memory-load.md
