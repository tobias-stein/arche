---
title: Generate endpoint — POST /api/generate
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Wire the generation engine into an HTTP endpoint:

- `POST /api/generate` — Accept generation request, run the full pipeline (blueprint selection → attribute rolling → affix selection → name composition → output assembly), return the result.
- If no seed is provided, generate a random one (using system entropy).
- If seed is provided, use it directly (enables reproducibility).
- The seed is always returned in the response.
- `X-Cache-Refresh: true` header: bypass the shared cache and load fresh data from DB for this single request (without updating the shared cache).
- 404 response with RFC 9457 when no blueprints match constraints.
- Returns `blueprint_attributes` as a JSON object keyed by attribute name.
- Returns `affix_attributes` as an ordered array.

The full response format matches `docs/design/api-spec.md`.

## Acceptance criteria

- [ ] Empty body generate: returns random result from current client's blueprints
- [ ] Archetype constraint: only blueprints of that archetype considered
- [ ] Attribute constraints: overlap semantics applied correctly
- [ ] Seed: deterministic output for same seed + same data
- [ ] No seed provided: random seed generated, returned in response
- [ ] Affix constraints: min/max, require, block all work
- [ ] No matching blueprints: 404 with Problem JSON
- [ ] `X-Cache-Refresh: true` forces fresh DB load for that request
- [ ] Response format matches API spec exactly
- [ ] Integration test: populate test data, call generate, verify output structure

## Blocked by

- arche-service/generation-blueprint-selection.md
- arche-service/generation-attribute-rolling.md
- arche-service/generation-affix-selection.md
- arche-service/generation-name-composition-output.md
- arche-service/auth-api-key-extractor.md
- arche-service/permission-matrix.md
