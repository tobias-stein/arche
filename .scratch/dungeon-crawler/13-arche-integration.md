---
title: "Arche API integration + seed script"
status: ready-for-agent
---

## What to build

Integrate with the Arche procedural generation service: API client, response parsing, and seed script for data population.

**Arche API client:**
- `POST /api/generate` with `Content-Type: application/json` and API key header.
- Request shape:
```typescript
interface GenerateRequest {
  archetype: string;            // "creature", "weapon", "armor", etc.
  constraints?: Record<string, any>;  // e.g. { level: { gte: 3, lte: 7 }, difficulty: { in: ["champion"] } }
  affixes?: {
    min_prefixes?: number;
    max_prefixes?: number;
    min_suffixes?: number;
    max_suffixes?: number;
  };
  seed?: number;
  count?: number;
}
```
- Response parsing: map Arche's `Thing` response to game entity types (CreatureState, ItemState).
- Error handling: if Arche is unavailable, fall back to locally-generated mock data for development.
- API key embedded in client config for local dev.

**Seed script (`scripts/seed.ts`):**
- TypeScript script run via `docker compose run --rm seed`.
- Generates Arche import JSON: `blueprints.json`, `affixes.json`, `global-meta-attributes.json`.
- Creates global meta attributes for all enum types: rarity, difficulty, creature_subtype, weapon_subtype, armor_subtype, shield_subtype, accessory_subtype, potion_type, spell_type, element.
- Generates creature blueprints combinatorially: 12 base types × 7 level bands × 4 difficulties = ~100 creatures.
- Generates item blueprints: weapons (10 × 4 rarities = 40), armor (6 slots × 4 rarities = 40), shields (3 × 4 = 12), accessories (2 × 4 = 8), potions (2 × 4 tiers = 8), spells (~8 base).
- Assigns affix pools to blueprints based on rarity/difficulty.
- Affixes tiered by level band (4 tiers: Weak 1–30, Strong 20–60, Greater 50–85, Mythic 75–100). Each generated 4 times with different names and scaled ranges.
- Difficulty distribution for creature pool: normal 50%, champion 25%, elite 15%, boss 10%.
- Item level bands match creature bands (7 bands, 1–100).

**Generation flow (for other slices to use):**
1. Game decides what to generate (room spawn, loot drop, etc.)
2. Game selects rarity/difficulty (weighted random via GAME_CONFIG)
3. Game selects level window: `playerLevel ± GAME_CONFIG.generationWindow.variance`
4. Game constructs Arche request → receives Thing → interprets as creature/item

## Prototype references

- `demos/dungeon-crawler/docs/DESIGN.md` — full content model documentation: archetypes, blueprint attributes per type, affix tables (4 tiers for creatures and items), rarity/difficulty mapping, level bands, creature/item distributions, generation flow. This is THE reference for seed script data.
- No prototype code exists for Arche integration — this is a new implementation.

## Acceptance criteria

- [ ] API client can call `POST /api/generate` and return a Thing
- [ ] Response parsing maps Thing to CreatureState / ItemState correctly
- [ ] Fallback mock data works when Arche is unavailable
- [ ] Seed script generates all blueprints, affixes, globals JSON files
- [ ] `docker compose run --rm seed` completes without errors
- [ ] Generated blueprints cover all 7 archetypes with correct attributes

## Blocked by

01-project-scaffold
