---
title: Overhaul seed script for fixed-level blueprint generation
status: ready-for-agent
---

## Problem

The current seed script generates blueprints using 7 overlapping level bands with range-type level attributes. This is complex and causes stat scaling issues. The new model uses fixed integer levels (`Single` value type) on each blueprint, with selection via flat `playerLevel ± variance` bands.

## Design

See `stat-reference-curve.md` for the shared stat formulas. See ADR for the fixed-level decision.

## Specification

### Level attribute

Every blueprint (creature, item, spell, potion) uses:
```json
"level": { "value_type": "single", "value": <integer> }
```

No ranges, no bands on the blueprint itself. The band exists only at generation-request time.

### Creature blueprints (~250 total, 2-3 per level across 100 levels)

- Each level (1-100) gets 2-3 creature blueprints
- Across all creatures, difficulty distribution:
  - normal: 50% (~125)
  - champion: 25% (~63)
  - elite: 15% (~37)
  - boss: 10% (~25)
- Creature subtypes spread across levels (no fixed band-to-subtype mapping, though some subtypes can cluster at certain levels for flavor — e.g. rats early, dragons late)
- Stats computed from `computeStats(level, creatureTemplate)` — see stat-reference-curve.md

### Item blueprints (~175 total, 1-2 per level across 100 levels)

- Each level gets 1-2 item blueprints (weapons, armor, shields, accessories)
- Across all items, rarity distribution:
  - common: 50%
  - uncommon: 25%
  - rare: 15%
  - legendary: 10%
- Item subtypes can be assigned per level arbitrarily — not every level needs every subtype
- Stats computed from `computeStats(level, itemTemplate)`

### Spell blueprints (~75 total, ~30% of item+spell pool, sparse across levels)

- Spells are archetype: `spell` (not `item`)
- Not every level has a spell — they're scattered across levels
- Rarity distribution (no common):
  - uncommon: 75%
  - rare: 15%
  - legendary: 10%
- Affix configuration per rarity:
  - uncommon: 1 prefix, 0 suffixes
  - rare: 1 prefix, 1 suffix
  - legendary: 2 prefixes, 1 suffix
- Prefix determines element/type (fire, ice, lightning, arcane, poison, holy)
- Suffix adds power/destruction/fortitude bonuses
- Stats from `computeStats(level, spellTemplate)`

### Potion blueprints (~8-16 total, spread across levels)

- Archetype: `potion`
- Fixed level, spread across 1-100
- Types: health, mana
- Potency from `computeStats(level, potionTemplate)`
- No rarity, no affixes

### Global meta attributes

Keep existing GMAs. No `level` GMA needed — level is inline.

### Affixes

Keep existing affix infrastructure. Add spell-specific prefixes for element types and suffixes for power bonuses. Map affix tier ranges to the new stat curve.

### Files to change

- `scripts/shared.ts` — stat curve constants, update config (remove `LEVEL_BANDS`, `BAND_SUBTYPES`, add per-level mapping)
- `scripts/seed.ts` — full rewrite of blueprint generation functions
- `scripts/validate.ts` — update validation to work with fixed levels

### Verification

- Script runs end-to-end against Arche API
- `scripts/validate.ts` confirms weight distributions match targets (within 10pp)
- Manual: generate 100 creatures at playerLevel=5, observe levels in [3,7] band