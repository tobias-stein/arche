---
title: "Player sometimes cannot move after game start or room transition (stuck)"
status: completed
---

## Description

Sometimes when starting a new game or entering a new room, the player character cannot move at all. The keyboard input has no effect. This appears to be related to the player spawning on a tile that overlaps with a creature or chest, or the player being placed on a wall tile.

## Current behaviour

**Game start:** Player spawns at `getPlayerSpawn()` which returns the centre of the room `{ x: 7, y: 5 }` for a 15×11 grid. Creatures and chests are then spawned using the same floor tile pool. While the `entryExclusionRadius` prevents spawns near the north door tile, the centre of the room (where the player spawns) is NOT excluded if the room is large enough. If a creature or chest is placed on the centre tile before the player, or if the tile pool accidentally includes the player's spawn position, the player can overlap with a creature.

**Room transition:** The player's new position is determined by `getDoorPos(oppositeDir(enterDir))`. This returns the tile just inside the room from the corresponding door. However:
- The exclusion in `pickSpawnPositions()`/`generateChestsForRoom()` always uses `getEntryTile()` which returns the **north door** tile, not the door the player actually enters from.
- If the player enters from the south, east, or west door, creatures/chests could be placed on or adjacent to the player's spawn position.

**Additional contributing factors:**
- `tryMovePlayer()` checks for `WALL` tiles and doors, but doesn't check if a creature or chest occupies the target tile.
- `startMove()` calls `checkChestInteraction()` and `checkEncounter()` after moving the player, but these are post-move checks that don't prevent the move.
- If the player is spawned on a tile occupied by a creature, `checkEncounter()` immediately triggers an encounter before the player can move.

## Expected behaviour

- Player spawn position should be excluded from creature and chest spawn pools.
- All four door entry positions (not just north) should be excluded.
- When entering a room, the player's spawn position should be checked against creature/chest positions and any that overlap should be repositioned.
- `tryMovePlayer()` should not allow moving onto a tile occupied by a stationary creature that is not stunned.

## Acceptance criteria

- [ ] Player can always move immediately on game start
- [ ] Player can always move immediately after room transition
- [ ] Player never spawns on the same tile as a creature or chest
- [ ] Player cannot walk into a creature's tile (should trigger encounter prompt from adjacent tile, not by overlapping)
