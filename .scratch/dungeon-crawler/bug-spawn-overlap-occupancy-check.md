---
title: "Creatures and chests can spawn on the same tile (no occupancy check)"
status: completed
---

## Description

Creatures and chests frequently spawn stacked on top of each other on the same floor tile. This happens because `pickSpawnPositions()` (creature spawner) and `generateChestsForRoom()` (chest spawner) independently select random floor tiles without checking whether a tile is already occupied by another entity.

## Current behaviour

In `DungeonScene.ts:201-224`, `spawnCreatures()` calls:

1. `generateCreaturesForRoom(tiles, ...)` → internally calls `pickSpawnPositions(tiles, count, entryTile)` which selects floor tiles from all available floor tiles, excluding only those within `entryExclusionRadius` of the entry tile (north door position).
2. `generateChestsForRoom(tiles, entryTile)` which independently selects floor tiles from all available floor tiles, using the same exclusion logic.

Neither function knows about the other's selections, so a chest can be placed on the same `{x, y}` as a creature, or two creatures can't overlap (since available tiles are removed from the pool), but a creature and a chest can.

Additionally, creatures can spawn on the tile where the player will enter the room (because the exclusion radius is based on the north door position, not the actual door the player enters through — see `getEntryTile()` which always returns `DOOR_N`).

## Expected behaviour

- `spawnCreatures()` should return both creatures and chests, OR `generateChestsForRoom` should receive the already-occupied positions and exclude them.
- No two entities (creature or chest) should occupy the same tile.
- The entry position (the door the player entered from) should be excluded from spawn positions for both creatures and chests, not just the north door.
- The player should never spawn on top of a creature or chest.

## Acceptance criteria

- [ ] Creatures and chests never occupy the same tile
- [ ] The player's entry position(s) (all four door positions) are excluded from spawn placement
- [ ] No entity spawns within 1 tile of any door the player could enter from
