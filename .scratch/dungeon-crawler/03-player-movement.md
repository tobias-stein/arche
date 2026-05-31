---
title: "Player movement + room transitions"
status: ready-for-agent
---

## What to build

Implement player movement within a room and door-based room transitions.

**Movement controls:**
- WASD and Arrow keys for directional movement.
- Hold-to-move: first step on keydown, subsequent steps at configurable interval (`GAME_CONFIG.movement.holdMoveInterval`, default 110ms). Track `heldDirs` Set — keydown adds, keyup removes. Last pressed direction wins.
- Click-to-navigate: click a floor tile → BFS pathfinds within the 15×11 room → player walks along path one tile per tick. Holding a direction key cancels auto-path.
- Walk speed: configurable (`GAME_CONFIG.movement.walkSpeed`, default 120ms per tile). Lerp between tile positions for smooth movement.

**Door transitions:**
- Stepping on a door tile triggers the fade transition.
- 350ms fade to black (`GAME_CONFIG.dungeon.transitionMs`), then player appears at the corresponding entry tile in the adjacent room.
- Entry door area: tiles within `GAME_CONFIG.creatureSpawn.entryExclusionRadius` (default 4) tiles of the entry door are NOT used for creature spawns (handled in slice 5, but the room data structure should track this).
- Room data tracked in GameState: current room ID, visited rooms Set, tile data for active room.

**Player rendering:**
- Programmatic Phaser Graphics (no sprites).
- Filled circle with radial gradient (`#e8dfd0` → `#d0c8b8`), margin 16% of tile size, 1px outline `#b0a898`.

## Prototype references

- `demos/dungeon-crawler/prototypes/room-generator-minimap/room-generator-minimap.html` — has the complete player movement implementation: held keys (heldDirs Set), click-to-navigate BFS pathfinding (`bfsPathfind()`), lerp-based smooth movement (`updateMovement()`), room transitions (`transitionToRoom()` with fade overlay), player rendering (blue glowing rounded rect — in the PRAD the player style was changed to a circle gradient, but the movement logic is directly reusable).
- `demos/dungeon-crawler/prototypes/room-layout-and-assets/room-layout-and-assets.html` — player rendering as a filled circle gradient (`drawPlayer()`). This is the canonical visual — use this over the blue rect from room-generator.
- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` — shows the exploration scene layout with player card and HUD.

## Acceptance criteria

- [ ] WASD/Arrow keys move player one tile per step
- [ ] Holding a key auto-repeats movement at 110ms interval
- [ ] Clicking a floor tile paths the player there via BFS
- [ ] Player walks through a door → 350ms fade → appears in adjacent room at correct entry tile
- [ ] Player card updates position in GameState on each move
- [ ] Multiple rooms can be traversed end-to-end

## Blocked by

02-dungeon-generation
