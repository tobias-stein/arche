---
title: "Arche API client"
status: completed
---

## What to build

The HTTP client that the game uses to call Arche's `POST /api/generate` endpoint. The seed script and content model are handled by slice 05 — this slice is purely the in-game integration layer.

**API client:**
- `POST /api/generate` with `Content-Type: application/json` and API key header.
- Request shape matching Arche's actual API:
```typescript
interface GenerateRequest {
  archetype?: string;
  constraints?: Record<string, { gte?: number; lte?: number; in?: string[] }>;
  affixes?: {
    min_prefixes?: number;
    max_prefixes?: number;
    min_suffixes?: number;
    max_suffixes?: number;
  };
  seed?: number;
}
```
- Response parsing: map Arche's `GenerateResponse` to game entity types (CreatureState, ItemState).
- Parse `name`, `blueprint_attributes`, `affix_attributes`, `name_parts` from the response.
- Error handling: if Arche is unavailable, fall back to locally-generated mock data for development.
- API key embedded in client config for local dev.

**Generation request helpers** (for use by creature spawning and loot system slices):

```typescript
// Creature spawn (non-boss room)
function buildCreatureRequest(playerLevel: number, excludeBoss?: boolean): GenerateRequest

// Loot drop
function buildLootRequest(creatureLevel: number): GenerateRequest

// Boss spawn
function buildBossRequest(playerLevel: number): GenerateRequest
```

**Mock data fallback:**
- When Arche is unreachable, generate deterministic mock creatures/items that still respect the level window and produce varied difficulties/rarities (simple weighted random in the game client)
- Mock data should produce a `CreatureState`/`ItemState` shaped identically to what Arche would return, so consuming code never forks on real vs mock

## Prototype references

- `crates/arche-types/src/generate.rs` — actual Arche GenerateRequest / GenerateResponse structs (for accurate HTTP payloads and field names)
- `crates/arche-service/src/generate.rs` — handler flow (to understand what the endpoint expects)
- `demos/dungeon-crawler/docs/PRD.md` — section 8 for the game-side API interface types

## Acceptance criteria

- [ ] API client can call `POST /api/generate` and return a Thing
- [ ] Response parsing correctly maps `blueprint_attributes` and `affix_attributes` to CreatureState / ItemState
- [ ] Helper functions build correct requests for creature spawn, boss spawn, and loot drop
- [ ] Fallback mock data produces valid CreatureState/ItemState objects
- [ ] Mock data respects level windows and produces varied difficulties/rarities
- [ ] Unavailable Arche causes no crashes — game works with mock data

## Blocked by

01-project-scaffold
