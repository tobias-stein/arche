---
title: Define and implement shared stat reference curve
status: ready-for-agent
---

## Problem

Stats (attack, defense, health) are currently scaled per level band with independent formulas for creatures vs items vs spells. This causes balance issues — low-level creatures can have stats that far exceed the player. A single shared reference curve ensures consistent scaling across all content.

## Design

See ADR (`docs/adr/`) for the fixed-level content model decision.

## Specification

### Reference curve formulas

Define a set of linear formulas that compute a baseline stat for any level:

```
referenceAttack(level)  = 5 + (level - 1) * 0.5
referenceDefense(level) = 2 + (level - 1) * 0.3
referenceHealth(level)  = 20 + (level - 1) * 2.0
```

At level 1: ATK=5, DEF=2, HP=20
At level 50: ATK=29.5, DEF=16.7, HP=118
At level 100: ATK=54.5, DEF=31.7, HP=218

### Archetype multipliers

Each archetype applies a multiplier against the reference to determine its actual stat:

| Archetype | Attack | Defense | Health | XP Reward |
|-----------|--------|---------|--------|-----------|
| Creature (normal) | 1.0x | 1.0x | 1.0x | 1.0x |
| Creature (champion) | 1.2x | 1.2x | 1.5x | 1.5x |
| Creature (elite) | 1.5x | 1.5x | 2.0x | 2.5x |
| Creature (boss) | 2.0x | 2.0x | 4.0x | 5.0x |
| Weapon | 0.8x ATK as damage | — | — | — |
| Armor | — | 0.7x as defense_bonus | — | — |
| Shield | — | 0.5x as defense_bonus | — | — |
| Accessory | 0.3x bonus (flex) | 0.3x bonus (flex) | 3x as hp bonus | — |
| Potion | — | — | 2x as heal value | — |
| Spell (damage) | 1.2x | — | — | — |
| Spell (heal) | — | — | 1.5x as heal | — |

### Implementation

Define a function `computeStats(level, archetypeTemplate)` that:
1. Computes reference values from the formulas
2. Applies the archetype multipliers
3. Adds ±10% variance for natural-feeling variety

### Files to create/change

- `scripts/shared.ts` — export `statCurve` and `STAT_MULTIPLIERS` constants
- `scripts/seed.ts` — use `computeStats()` in `buildCreatureBlueprint()`, `buildItemBlueprint()`, `buildSpellBlueprint()`, `buildPotionBlueprint()`
- `src/config.ts` or `src/api/response-parser.ts` — ensure consistency if game-side stat display needs the curve

### Verification

- Seed generates blueprints with predictable stats per level
- Level 1 normal creature: ~ATK=5, DEF=2, HP=20
- Level 1 weapon: ~4 damage
- Level 100 boss: ~ATK=109, DEF=63, HP=872