# ADR 0005: Attribute Ordering — JSONB Map + Separate Order Array

**Date:** 2026-05-22
**Status:** Accepted

## Context

Blueprint attributes need a user-defined display order (e.g., damage first, defense second, weight third). The generation response also outputs attributes in this order. The existing `attributes` column is a JSONB map (`{ "key": { ... } }`), which inherently has no key ordering — JavaScript objects preserve insertion order, but PostgreSQL JSONB sorts keys alphabetically.

Three approaches were considered for adding ordering:
1. Store attributes as a JSONB **ordered array** (list of key-value pairs)
2. Keep the JSONB **map** + add a separate `TEXT[]` column for key order
3. Alphabetical sort only (no custom ordering)

## Decision

Keep the JSONB **map** for attribute storage and add a **separate `attribute_order TEXT[]` column** on the `blueprints` table.

The `attribute_order` array lists attribute keys in display order. The application enforces at write time that:
- All keys in `attribute_order` exist in the `attributes` map
- All keys in `attributes` appear in `attribute_order`
- No duplicates in `attribute_order`

## Consequences

**Positive:**
- O(1) attribute lookup preserved for generation and constraint matching (`attributes["damage"]`)
- GIN index on `attributes` JSONB continues to work unchanged for queries like `WHERE attributes @> '{"damage": ...}'`
- No risk of duplicate keys (map inherently enforces uniqueness)
- Order is explicit, serializable, and independent of JSONB key ordering quirks
- Easy to reorder — just reorder the `TEXT[]` array, no need to rewrite the entire `attributes` JSONB

**Negative:**
- Two fields to keep in sync — the application must validate consistency on every write
- Slightly larger storage (duplicated key names in both the map and the array)
- More complex migration path if the schema changes later (both fields must be updated atomically)

## Alternatives Considered

- **Attribute array** — store `[ { "key": "damage", ... }, { "key": "defense", ... } ]`. Unifies definition and order but requires O(n) lookups and application-level duplicate key enforcement, and needs a different GIN query syntax.
- **Alphabetical only** — simplest, but unacceptable for UX. Users want to control display order (especially for generated output).
- **Integer sort key per attribute** — add an `order` field inside each map entry. Ordering requires scanning all entries. Easy to create gaps/ties. No clear advantage over `TEXT[]`.
