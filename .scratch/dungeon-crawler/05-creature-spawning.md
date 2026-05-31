---
title: "Creature spawning + aggro/chase"
status: ready-for-agent
---

## What to build

Spawn creatures in rooms with difficulty-based rendering, aggro ranges, chase behavior, and encounter trigger.

**Spawn:**
- Each room spawns `rand(GAME_CONFIG.creatureSpawn.minPerRoom, GAME_CONFIG.creatureSpawn.maxPerRoom)` creatures (default 1–4).
- Boss room: exactly 1 boss creature.
- Spawn on random floor tiles, minimum `GAME_CONFIG.creatureSpawn.entryExclusionRadius` (default 4) tiles from the entry door tile.
- Soft constraint: try to avoid overlapping aggro ranges between spawned creatures.
- Difficulty distribution: weighted random from `GAME_CONFIG.creatureSpawn.difficultyWeights` (normal 50%, champion 25%, elite 15%, boss 10%).
- Creatures respawn when re-entering a room.

**Rendering:**
- Programmatic Phaser Graphics (no sprites).
- Triangle shape (point-up), margin 20%, half-width 57.7% of height.
- Linear gradient (color lightened 50 → base color), 2px stroke base color, 1px stroke darkened 40.
- "!" symbol in `#f0e8d8` centered, font size 30% of tile.
- Colors by difficulty: normal `#889096`, champion `#d4883a`, elite `#c83838`, boss `#7a388a`.

**Behavior:**
- Stationary at spawn position until aggro'd. No patrol/wander.
- Aggro ranges (Chebyshev tile radius): from `GAME_CONFIG.aggroRanges` (normal=1, champion=2, elite=3, boss=3).
- When player enters aggro range: creature starts chasing.
- Only the nearest aggro'd creature chases at any time. Others remain stationary.
- Chase speed: `GAME_CONFIG.creatureChaseSpeed` (default 0.5 = half player speed, moves every 2nd step).
- Encounter trigger: when player walkable tile overlaps creature tile (same-tile bump), emit `encounter:started` event.

**Post-flee:**
- Creature stays at encounter position. Gains a `GAME_CONFIG.fleeStunDuration` (default 1500ms) stun window — no aggro, no movement.
- After stun, creature resumes stationary behavior at current position.

## Prototype references

- `demos/dungeon-crawler/prototypes/room-layout-and-assets/room-layout-and-assets.html` — enemy rendering (`drawEnemy()` function) with the triangle shape, gradient fill, difficulty colors (`EC` object), "!" symbol, and margin calculations. Translate these exact drawing calls from Canvas 2D to Phaser Graphics.
- `demos/dungeon-crawler/prototypes/room-generator-minimap/room-generator-minimap.html` — the room state management (player position, tile grid, door detection) is the foundation creatures will sit on top of. No creature behavior exists in the prototypes — this is a new implementation based on the PRD specs.

## Acceptance criteria

- [ ] 1–4 creatures spawn in each room on random floor tiles (excluding 4-tile door zone)
- [ ] Creatures are stationary until player enters aggro range
- [ ] Aggro ranges vary by difficulty (1/2/3/3)
- [ ] Nearest creature chases at half player speed; others remain
- [ ] Player touching a creature triggers encounter event
- [ ] Fleeing leaves creature stunned for 1.5s, then resumes stationary
- [ ] Re-entering room respawns creatures
- [ ] Boss room has exactly 1 creature with boss color

## Blocked by

03-player-movement
