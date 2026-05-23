# ADR 0004: Meta Attributes — Inline Definitions + Global $ref_id

**Date:** 2026-05-22
**Status:** Accepted

## Context

Many blueprints share common attributes (e.g., `rarity` as an enum). Defining the same attribute inline on every blueprint is tedious and error-prone. But some blueprints need custom overrides (e.g., a unique sword with a different rarity scale).

## Decision

Two ways to specify a meta attribute on a blueprint or affix:

1. **Inline:** Full attribute definition embedded directly:
```json
"rarity": {
  "value_type": "enum",
  "values": ["common", "uncommon", "rare", "legendary"]
}
```

2. **Reference to global:** A `$ref_id` pointing to a global meta attribute:
```json
"rarity": {
  "$ref_id": "uuid-of-rarity"
}
```

**Resolution rules:**
- At generation time, each attribute is resolved independently
- If a key has `$ref_id`, resolve from the global pool
- If a key has inline definition, use that directly (global is ignored for this key)
- A key cannot have both `$ref_id` and inline fields — that's a validation error
- Global updates propagate automatically since resolution happens at generation time

## Consequences

**Positive:**
- DRY for common attributes (define once, reference everywhere)
- Flexibility for edge cases (inline override when needed)
- No schema migration needed when adding a new attribute type

**Negative:**
- Attribute resolution adds a lookup step during generation (minor CPU cost)
- Two ways to do the same thing confuses new users — the frontend must make the choice clear
- Deleting a global attribute requires cascading $ref_id removal (handled by force delete)
- A blueprint's effective attribute set is not visible without resolving $ref_id — caching must handle this

## Alternatives Considered

- **All inline** — simple but tedious. Every blueprint repeats the same definitions. Changing a common attribute requires updating N blueprints.
- **All global references** — clean but rigid. Every variation requires a new global attribute. No per-blueprint overrides.
- **Inheritance / blueprint hierarchy** — a blueprint inherits from a "base" and overrides fields. More complex model, harder to reason about at generation time. Overkill for this use case.
