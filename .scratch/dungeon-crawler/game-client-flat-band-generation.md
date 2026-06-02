---
title: Update game client generation flow for flat band + single-value level
status: ready-for-agent
---

## Problem

The game client currently maps player level to one of 7 predefined bands and sends `{ level: { gte: band.min, lte: band.max } }` to Arche. With the new fixed-level content model, it should use a simple `playerLevel ± variance` flat band and handle single-value level responses.

## Design

The Arche service now supports `gte`/`lte` constraints on `Single` value attributes, and the seed script generates blueprints with fixed integer levels.

## Specification

### Remove band mapping

Replace `findBand(playerLevel)` with direct computation:
```typescript
const band = {
  gte: playerLevel - GAME_CONFIG.generationWindow.creatureLevelVariance,
  lte: playerLevel + GAME_CONFIG.generationWindow.creatureLevelVariance,
}
```

Clamp to 1-100 bounds.

### No explicit affix constraints needed

Do NOT specify `affixes` in generate requests. Arche's `affix_selection.rs` falls back to the blueprint's own `min_prefixes`/`max_prefixes`/`min_suffixes`/`max_suffixes` when the request omits them. The seed script already bakes the correct affix counts per difficulty/rarity into each blueprint. Explicit constraints are redundant and introduce drift risk.

### Generation requests

Non-boss creature request:
```typescript
{
  archetype: 'creature',
  constraints: {
    level: { gte: band.gte, lte: band.lte },
    difficulty: { in: ['normal', 'champion', 'elite'] },
  },
}
```

Boss creature request (boss rooms only):
```typescript
{
  archetype: 'creature',
  constraints: {
    level: { gte: band.gte, lte: band.lte },
    difficulty: { in: ['boss'] },
  },
}
```

Loot drop request:
```typescript
{
  constraints: {
    level: { gte: band.gte, lte: band.lte },
  },
}
```

### Response parsing

Update `response-parser.ts` to handle `level` as a single number instead of rolling from a range:
```typescript
const level = getNumber(blueprint_attributes, 'level', 1)
```
(The rolled value from Arche's `Single` payload is the blueprint's fixed value, so this already works — just verify the parsing.)

### Stat display

Ensure the game reads creature/item stats directly from the generated response (no game-side stat computation needed — the seed bakes correct stats into the blueprint stat attributes).

### Files to change

- `src/api/request-builders.ts` — replace `findBand()` with flat band
- `src/api/response-parser.ts` — verify `level` parsing
- `src/config.ts` — ensure `generationWindow` config matches new structure (already has `creatureLevelVariance: 2`, `itemLevelVariance: 2` — may just work)
- `src/GameState.ts` — verify stat handling

### Verification

- Game starts, spawns creatures with levels in playerLevel ± 2 window
- Loot drops with levels in creatureLevel ± 2 window
- No band-edge artifacts (e.g. no creatures below level 1 or above 100)
- Removing `findBand()` code and its 7-band mapping