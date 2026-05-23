---
title: arche-types — request/response types
status: ready-for-agent
---

## Parent

arche-types shared library (see `.scratch/arche-types/prd.md`)

## What to build

Define all API request and response types in `arche-types`. These map one-to-one to the API spec endpoints in `docs/design/api-spec.md`.

Includes:
- Generate request (archetype, seed, constraints, affix min/max/require/block) and response (seed, name, name_parts, blueprint_id, blueprint_attributes, affix_attributes)
- CRUD request/response types for blueprints, affixes, global meta attributes, clients, API keys
- Batch operation request types (batch delete IDs, batch assign, batch edit)
- Export request (client_ids, include_api_keys, include_audit_log, inline_global_refs)
- Import response types (success, conflict details, import token, resolution request)
- Paginated list response wrapper with `data`, `next_cursor`, `total`
- RFC 9457 Problem JSON struct
- Attribute value payload types (single, enum, range, string, boolean) with distribution config

All types implement `Serialize`/`Deserialize` with camelCase naming.

## Acceptance criteria

- [ ] Every API endpoint in `docs/design/api-spec.md` has a corresponding request/response type
- [ ] Attribute payload variants are modeled as an enum or with serde tag
- [ ] Distribution config types (uniform, normal, exponential) are modeled
- [ ] Constraint types (gte, lte, in, contains, eq) for generate request are modeled
- [ ] `cargo test` passes
- [ ] Types round-trip through JSON serialization correctly

## Blocked by

- arche-types/domain-structs-enums.md
