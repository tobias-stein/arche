# ADR 0002: Constraint Matching — Overlap Semantics

**Date:** 2026-05-22
**Status:** Accepted

## Context

The generate endpoint accepts attribute constraints (e.g., `damage: { gte: 15 }`). Blueprints define attribute ranges (e.g., `damage: [10, 23]`). The question is: when does a blueprint match a constraint?

## Decision

Use **overlap semantics**: a blueprint matches if its attribute range **overlaps** with the constraint range.

Example:
- Blueprint: `damage: [10, 23]`
- Constraint: `damage: { gte: 15 }`
- Result: **match** (ranges `[10, 23]` and `[15, ∞)` overlap)

Example:
- Blueprint: `damage: [5, 10]`
- Constraint: `damage: { gte: 15 }`
- Result: **no match** (ranges `[5, 10]` and `[15, ∞)` do not overlap)

## Consequences

**Positive:**
- Intuitive for game designers — "this sword *can* do 15+ damage, so it qualifies"
- Permissive matching → more blueprints available for generation
- The actual rolled value may still be below the constraint threshold, but that's the caller's responsibility to handle

**Negative:**
- Non-deterministic from the caller's perspective — a blueprint matches but the rolled value might not satisfy the constraint
- Callers who want guarantees must roll multiple times or post-filter

## Alternatives Considered

- **Fully-contained matching** — blueprint's range must be entirely within the constraint. Too restrictive — excludes blueprints whose range barely overlaps.
- **Midpoint matching** — blueprint matches if the midpoint of its range satisfies the constraint. Introduces a hidden assumption about distribution shape.
- **No constraint matching** — always return random, let the caller filter. Simple but defeats the purpose of the endpoint.
