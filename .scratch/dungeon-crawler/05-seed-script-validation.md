---
title: "Seed script + distribution validation"
status: completed
---

## What to build

A standalone TypeScript seed script (`scripts/seed.ts`) that generates valid Arche import JSON, loads it into the service, then runs a statistical validation suite to prove the blueprint weights produce the expected distributions.

This slice runs before any game code. It validates that Arche's generate endpoint returns creatures and items matching the configured level windows and rarity/difficulty targets.

### Seed Script

Generates three Arche import files:

- `global-meta-attributes.json` — all enum types: rarity, difficulty, creature_subtype, weapon_subtype, armor_subtype, shield_subtype, accessory_subtype, potion_type, spell_type, element
- `blueprints.json` — all creature and item blueprints with weighted selection, level bands, affix configs, and attribute ranges
- `affixes.json` — all affixes tiered by level band (Weak 1–30, Strong 20–60, Greater 50–85, Mythic 75–100)

Blueprint weights calibrate the target distributions:

| Difficulty / Rarity | Target frequency | Weight |
|---|---|---|
| normal / common | ~50% | 1.0 |
| champion / uncommon | ~25% | 0.5–0.7 |
| elite / rare | ~15% | 0.3–0.4 |
| boss / legendary | ~10% | 0.1–0.2 |

Creature coverage:
- 12 base types (goblin, skeleton, slime, bat, rat, spider, wolf, ghost, orc, troll, demon, dragon)
- 7 level bands (1–10, 5–20, 15–35, 30–50, 45–70, 60–85, 80–100)
- 4 difficulties (normal, champion, elite, boss)
- Boss variants per level band

Item coverage:
- Weapons (10 subtypes × 4 rarities)
- Armor (6 slots × 4 rarities)
- Shields (3 × 4 rarities)
- Accessories (ring, amulet × 4 rarities)
- Potions (health, mana × 4 tier bands)
- Spells (~8 base with element affixes)

Affix pools:
- Prefix + suffix affixes per rarity/difficulty tier (see DESIGN.md)
- Affix counts on blueprints: normal/common = 0, champion/uncommon = 1 prefix, elite/rare = 1+1, boss/legendary = 2+1
- Affix level tiers scaled to blueprint level band

### Distribution Validation

A validation script (`scripts/validate.ts`) that runs after the seed data is loaded into Arche:

**Creature spawn validation** (repeat for player levels 5, 25, 50, 75):
```
for each playerLevel in [5, 25, 50, 75]:
  levelWindow = playerLevel ± 2
  results = []
  for i in 0..100:
    response = POST /api/generate {
      constraints: {
        level: { gte: levelWindow.gte, lte: levelWindow.lte },
        difficulty: { in: ["normal", "champion", "elite"] }
      }
    }
    results.push(response)

  assert: all(results, r => r.level >= levelWindow.gte && r.level <= levelWindow.lte)
  assert: distribution(results, "difficulty") matches targets within ±10%
    // e.g. normal ∈ [40%, 60%], champion ∈ [15%, 35%], elite ∈ [5%, 25%]
```

**Loot drop validation** (repeat for creature levels 5, 25, 50, 75):
```
for each creatureLevel in [5, 25, 50, 75]:
  levelWindow = creatureLevel ± 2
  results = []
  for i in 0..100:
    response = POST /api/generate {
      constraints: {
        level: { gte: levelWindow.gte, lte: levelWindow.lte }
      }
      // no archetype constraint — Arche picks from all item blueprints
    }
    results.push(response)

  assert: all(results, r => r.level >= levelWindow.gte && r.level <= levelWindow.lte)
  assert: distribution(results, "rarity") matches targets within ±10%
  // Also valid item types appear (weapon, armor, potion, etc.)
```

**Validations that must pass:**
1. Every generated creature's level falls within `playerLevel ± creatureLevelVariance`
2. Every generated item's level falls within `creatureLevel ± itemLevelVariance`
3. Creature difficulty distribution matches seed script weights within ±10% at each player level
4. Item rarity distribution matches seed script weights within ±10% at each creature level
5. No generation errors from Arche (all 100 requests succeed)
6. Generated items cover multiple archetypes (weapon, armor, potion, spell, accessory)
7. Generated creatures cover multiple subtypes and difficulties

### Execution

```
docker compose run --rm seed          # generates import JSON + loads into Arche
docker compose run --rm validate      # runs 400+ generate calls, reports pass/fail
```

Both scripts share a types module for blueprint templates, config, and weight definitions.

### Config-Driven Weights

The seed script defines target weights in a config object at the top:

```typescript
const SEED_CONFIG = {
  creatureWeights: {
    normal: { weight: 1.0 },
    champion: { weight: 0.6 },
    elite: { weight: 0.35 },
    boss: { weight: 0.15 },
  },
  rarityWeights: {
    common: { weight: 1.0 },
    uncommon: { weight: 0.6 },
    rare: { weight: 0.35 },
    legendary: { weight: 0.15 },
  },
  levelVariance: 2,
};
```

Changing these weights and re-running produces new distributions. The validation script must reflect updates automatically.

### Edge cases

- Insufficient blueprints at a given level band + difficulty intersection should not break generation (Arche should pick from the next-best match or log a warning)
- Running validate before seed has been loaded should fail with a clear error message
- Statistical variance at small sample sizes (100 rolls) means ±10% tolerance is expected — if a test is flaky within tolerance, consider increasing sample size to 500

## Prototype references

No prototype code exists — this is a new implementation from scratch. Reference:
- `demos/dungeon-crawler/docs/DESIGN.md` — full content model: archetypes, blueprint attributes per type, affix tables (4 tiers for creatures and items), rarity/difficulty mapping, level bands, creature/item distributions, generation flow
- `demos/dungeon-crawler/docs/PRD.md` - generation flow, GAME_CONFIG for variance values
- `crates/arche-types/src/generate.rs` — actual Arche GenerateRequest / GenerateResponse structs (for accurate HTTP payloads)

## Acceptance criteria

- [ ] `global-meta-attributes.json` created with all enum types
- [ ] `blueprints.json` created with ~100 creatures and ~100 items across correct level bands
- [ ] `affixes.json` created with 4-tier affix sets
- [ ] Seed script loads data into Arche (via API or direct DB import)
- [ ] Validation runs 100 generate calls per player/creature level (5, 25, 50, 75)
- [ ] All generated creatures respect level window (variance ± 2)
- [ ] All generated items respect level window (variance ± 2)
- [ ] Creature difficulty distribution matches weights within ±10%
- [ ] Item rarity distribution matches weights within ±10%
- [ ] Validation exits with code 0 on pass, non-zero on failure

## Blocked by

01-project-scaffold
