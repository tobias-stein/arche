# ADR 0006: Fixed-Level Blueprint Model with Shared Stat Curve

**Date:** 2026-06-02
**Status:** Accepted

## Context

The dungeon-crawler demo originally used range-valued `level` attributes with overlapping level bands (e.g., band 1: 1-10, band 2: 5-20). At generation time, Arche rolled the level within the band, and the game mapped player level to a band to constrain selection. This design had several problems:

1. **Stat scaling drift:** Creature and item stats scaled independently per archetype, using different formulas. Low-level creatures could wildly exceed the player's effective power because there was no shared reference curve.
2. **Overlapping bands obscured level meaning:** A creature in band 2 (5-20) might roll level 20, while a creature in band 3 (15-35) might roll level 15 — the same level could come from different bands.
3. **Band complexity:** The 7-band system with `findBand()` mapping added game-side complexity and introduced edge cases at band boundaries.
4. **Spells disconnected from level system:** Spells had a flat 1-100 range with no meaningful level scaling, making them feel detached from progression.

A simpler model was needed: each blueprint has a fixed integer level, and selection uses a flat `playerLevel ± variance` band.

## Decision

### 1. Level as inline Single, not Range

Every blueprint (creature, item, spell, potion) defines `level` as an inline `Single` attribute with a fixed integer value:

```json
"level": { "value_type": "single", "value": 42 }
```

No rolling, no distribution. The level is the blueprint's fixed power tier.

### 2. Arche supports gte/lte on Single values

The constraint matching logic in Arche is extended so that `{ level: { gte: X, lte: Y } }` works against `Single` payloads, not just `Range`. This enables flat-band selection without changing the Arche API contract.

### 3. Flat band selection

The game computes `{ gte: playerLevel - variance, lte: playerLevel + variance }` (clamped to 1-100) and sends it as the level constraint. No band mapping, no overlap logic. Arche's weighted random selection picks from all matching blueprints uniformly within the band.

### 4. Shared stat reference curve

All stats derive from a single set of linear formulas:

```
referenceAttack(level)  = 5 + (level - 1) * 0.5
referenceDefense(level) = 2 + (level - 1) * 0.3
referenceHealth(level)  = 20 + (level - 1) * 2.0
```

Each archetype applies a multiplier to the reference (e.g., normal creature = 1.0x, champion = 1.2x, boss = 2.0x attack). This guarantees that a level-N creature and a level-N item are always in the same power band.

### 5. Spells as first-class archetype with rarity

Spells get their own `spell` archetype (separate from `item`), with rarity distribution (uncommon 75%, rare 15%, legendary 10%, no common). Affixes on spells determine element (prefix) and power bonuses (suffix), matching the same tier system as items.

### 6. Blueprint distribution

- **Creatures:** ~250 total, 2-3 per level across 1-100. Difficulty: normal 50%, champion 25%, elite 15%, boss 10%.
- **Items:** ~175 total, 1-2 per level. Rarity: common 50%, uncommon 25%, rare 15%, legendary 10%.
- **Spells:** ~75 total (~30% of item+spell pool), sparse across levels (not one per level). Rarity: uncommon 75%, rare 15%, legendary 10%.
- **Potions:** ~8-16 total, spread across levels. No rarity.

## Consequences

**Positive:**
- Predictable, level-appropriate stats everywhere — a level 5 weapon and a level 5 creature are always in the same power band
- Simpler generation flow: no `findBand()`, no band overlap edge cases, no range rolling
- Spells integrate naturally into the level progression system
- The shared stat curve means balancing one archetype implicitly affects all archetypes consistently
- Fewer blueprint types to reason about — less combinatorial explosion in the seed script

**Negative:**
- Each level needs its own blueprints → seed script must generate ~500 blueprints (up from ~200)
- Less variety per generation call (only ~2-3 options per exact level per archetype, though the band gives ~10-15 total)
- Arche needs the gte/lte-for-Single change (small, but requires a server update)
- Existing seed data is incompatible — requires full re-seed

## Alternatives Considered

- **Keep ranges, fix formulas independently:** Would solve stat imbalance but retain all the band complexity. Misses the opportunity to simplify.
- **Keep ranges with min=max for single semantics:** Works but needlessly complex — why store a range when you mean a single value?
- **Game-side normal distribution sampling:** Using truncated normal to pick a target level before calling Arche with `eq`. Rejected because it narrows the pool to 2-3 options per call instead of the full band.
- **Level as a Global Meta Attribute (`$ref_id`):** Would force all blueprints to share the same value — doesn't work.

## Related

- ADR 0002: Constraint Overlap Semantics (gte/lte matching on Range) — extended here to also apply to Single
- ADR 0004: Meta Attributes — Inline vs Ref (level remains inline, not a GMA ref)
